#![cfg_attr(coverage_nightly, coverage(off))]
#![cfg(test)]

use crate::scheduler::Context;
use crate::scheduler::builder::{TaskBuilder, parallel, pipeline, race, task};
use crate::scheduler::model::TaskSnapshot;
use crate::scheduler::model::TaskState;
use crate::scheduler::sync::RaceContext;
use crate::scheduler::*;
use anyhow::anyhow;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tokio_util::sync::CancellationToken;

mod execution_specs {
    use super::*;

    #[tokio::test]
    async fn should_execute_single_task_successfully() {
        let scheduler = Scheduler::new(1);
        let task = task("simple", |_| async { Ok(42) })
            .with_weight(10)
            .hidden_in_view();

        assert_eq!(task.weight(), 10);
        assert!(task.is_hidden_in_view());

        let result = scheduler.run(task).await;

        assert_eq!(result.unwrap(), 42);

    }
    #[tokio::test]
    async fn task_with_ctx_should_execute_successfully_too() {
        let scheduler = Scheduler::new(1);
        let task = task_with_ctx("simple", |_, _| async { Ok(42) })
            .with_weight(10)
            .hidden_in_view();

        assert_eq!(task.weight(), 10);
        assert!(task.is_hidden_in_view());

        let result = scheduler.run(task).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn should_pass_data_through_pipeline() {
        let scheduler = Scheduler::new(1);
        let pipeline_task = pipeline("calc_pipeline")
            .first(task("add_one", |num: i32| async move { Ok(num + 1) }))
            .then(task("double", |num: i32| async move { Ok(num * 2) }))
            .build();

        let ctx = Context {
            race_ctx: None,
            semaphore: scheduler.semaphore.clone(),
            registry: scheduler.registry.clone(),
            parent_id: None,
            cancel_token: CancellationToken::new(),
        };

        let result = pipeline_task.run(10, ctx).await;

        assert_eq!(result.unwrap(), 22);
    }

    #[tokio::test]
    async fn should_support_builder_chaining_syntax() {
        let scheduler = Scheduler::new(1);
        let chain_task = TaskBuilder::new(task("start", |_| async { Ok(1) }))
            .then(task("end", |n| async move { Ok(n + 1) }))
            .build();

        let result = scheduler.run(chain_task).await;

        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn should_correctly_determine_chain_visibility() {
        let visible_chain = pipeline("visible_chain")
            .first(task("visible", |_: ()| async { Ok(()) }))
            .then(task("visible_tail", |_| async { Ok(()) }))
            .build();
        assert!(!visible_chain.is_hidden_in_view());

        let hidden_chain = pipeline("hidden_chain")
            .first(task("hidden", |_: ()| async { Ok(()) }).hidden_in_view())
            .then(task("hidden_tail", |_| async { Ok(()) }).hidden_in_view())
            .build();
        assert!(hidden_chain.is_hidden_in_view());

        let head_hidden_chain = pipeline("mixed_chain")
            .first(task("hidden", |_: ()| async { Ok(()) }).hidden_in_view())
            .then(task("visible", |_| async { Ok(()) }))
            .build();
        assert!(!head_hidden_chain.is_hidden_in_view());

        let tail_hidden_chain = pipeline("mixed_chain_2")
            .first(task("visible", |_: ()| async { Ok(()) }))
            .then(task("hidden", |_| async { Ok(()) }).hidden_in_view())
            .build();
        assert!(!tail_hidden_chain.is_hidden_in_view());
    }
}

mod cancellation_specs {
    use super::*;

    #[tokio::test]
    async fn should_cancel_task_waiting_for_permit() {
        let scheduler = Scheduler::new(1);

        let _blocker = scheduler.run(task("blocker", |_| async {
            sleep(Duration::from_millis(100)).await;
            Ok(())
        }));

        let semaphore = Arc::new(tokio::sync::Semaphore::new(1));

        let _permit = semaphore.clone().try_acquire_owned().unwrap();

        let token = tokio_util::sync::CancellationToken::new();
        let ctx = Context {
            race_ctx: None,
            semaphore: semaphore.clone(),
            registry: Arc::new(dashmap::DashMap::new()),
            parent_id: None,
            cancel_token: token.clone(),
        };

        let task = task("waiting_task", |_| async { Ok("I should not run") });

        token.cancel();

        let result = task.run((), ctx).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Task cancelled before start"
        );
    }

    #[tokio::test]
    async fn should_cancel_running_task() {
        let ctx = Context {
            race_ctx: None,
            semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
            registry: Arc::new(dashmap::DashMap::new()),
            parent_id: None,
            cancel_token: tokio_util::sync::CancellationToken::new(),
        };
        let token_clone = ctx.cancel_token.clone();

        let task = task("long_task", move |_| {
            let token = token_clone.clone();
            async move {
                sleep(Duration::from_millis(10)).await;
                token.cancel();
                sleep(Duration::from_millis(100)).await;
                Ok("Finished")
            }
        });

        let result = task.run((), ctx).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }
}

mod priority_specs {
    use super::*;

    #[tokio::test]
    async fn critical_task_should_bypass_concurrency_limit() {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(1));
        let _permit = semaphore.clone().try_acquire_owned().unwrap();

        let ctx = Context {
            race_ctx: None,
            semaphore: semaphore.clone(),
            registry: Arc::new(dashmap::DashMap::new()),
            parent_id: None,
            cancel_token: tokio_util::sync::CancellationToken::new(),
        };

        let critical_task = task("vip", |_| async { Ok("VIP Pass") }).critical();

        let result = timeout(Duration::from_millis(50), critical_task.run((), ctx)).await;

        assert!(result.is_ok(), "Task timed out, meaning it was blocked!");
        assert_eq!(result.unwrap().unwrap(), "VIP Pass");
    }
}

mod concurrency_specs {
    use super::*;

    #[tokio::test]
    async fn parallel_tasks_should_aggregate_all_results() {
        let scheduler = Scheduler::new(4);
        let parallel_group = parallel("parallel_group")
            .add(task("t1", |_| async { Ok("A") }))
            .add(task("t2", |_| async { Ok("B") }))
            .build();

        let result = scheduler.run(parallel_group).await;

        let mut results = result.unwrap();
        results.sort();
        assert_eq!(results, vec!["A", "B"]);
    }

    #[tokio::test]
    async fn parallel_group_should_extend_dynamically() {
        let scheduler = Scheduler::new(4);
        let dynamic_tasks = (2..=3)
            .map(|i| task(format!("t{i}"), move |_| async move { Ok(i) }))
            .collect::<Vec<_>>();

        let group = parallel("extend_group")
            .add(task("t1", |_| async { Ok(1) }))
            .extend(dynamic_tasks)
            .build();

        let result = scheduler.run(group).await;

        let mut results = result.unwrap();
        results.sort();
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn race_should_return_result_of_fastest_winner() {
        let scheduler = Scheduler::new(2);
        let race_group = race("race_group")
            .add(task("slow", |_| async {
                sleep(Duration::from_millis(100)).await;
                Ok("slow")
            }))
            .add(task("fast", |_| async {
                sleep(Duration::from_millis(10)).await;
                Ok("fast")
            }))
            .build();

        let result = scheduler.run(race_group).await;

        assert_eq!(result.unwrap(), "fast");
    }
}

mod reliability_specs {
    use super::*;

    #[tokio::test]
    async fn parallel_should_fail_if_any_child_fails() {
        let scheduler = Scheduler::new(4);
        let parallel_group = parallel("parallel_fail")
            .add(task("ok", |_| async { Ok(()) }))
            .add(task("fail", |_| async { Err(anyhow!("oops")) }))
            .build();

        let result = scheduler.run(parallel_group).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Parallel task 1 failed: oops"
        );
    }

    #[tokio::test]
    async fn race_should_fail_only_if_all_children_fail() {
        let scheduler = Scheduler::new(2);
        let race_group = race("race_fail")
            .add(task("f1", |_| async {
                Result::<(), _>::Err(anyhow!("e1"))
            }))
            .add(task("f2", |_| async { Err(anyhow!("e2")) }))
            .build();

        let result = scheduler.run(race_group).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("All race tasks failed")
        );
    }

    #[tokio::test]
    async fn should_catch_panic_in_tasks() {
        let scheduler = Scheduler::new(4);
        let group = parallel("panic_group")
            .add(task("ok", |_| async { Ok(()) }))
            .add(task("panic_task", |_| async {
                panic!("intentional panic for coverage");
            }))
            .build();

        let result = scheduler.run(group).await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Task panic"));
    }
}

mod internal_mechanics_specs {
    use super::*;

    #[tokio::test]
    async fn should_retrieve_task_snapshot_by_id() {
        let scheduler = Scheduler::new(1);
        let task = task("view_test", |_| async { Ok(()) });
        let task_id = task.id();

        let _ = scheduler.run(task).await;

        assert!(!scheduler.tree_view().is_empty());

        let snapshot = scheduler.get_snapshot(task_id);
        assert!(snapshot.is_some());
        assert_eq!(snapshot.unwrap().name, "view_test");

        assert!(scheduler.get_snapshot(uuid::Uuid::new_v4()).is_none());
    }

    #[test]
    fn should_calculate_progress_weighted_average() {
        let parent = create_snapshot(TaskState::Running, 1);
        let children = vec![
            create_node(TaskState::Finished, 1.0, 1),
            create_node(TaskState::Pending, 0.0, 1),
        ];

        let progress = parent.calculate_effective_progress(&children);

        assert_eq!(progress, 0.5);
    }

    #[test]
    fn should_handle_zero_weight_children_edge_cases() {
        let child_zero_weight = create_node(TaskState::Pending, 0.0, 0);
        let children = vec![child_zero_weight];

        let parent_running = create_snapshot(TaskState::Running, 1);
        assert_eq!(parent_running.calculate_effective_progress(&children), 0.0);

        let parent_finished = create_snapshot(TaskState::Finished, 1);
        assert_eq!(parent_finished.calculate_effective_progress(&children), 1.0);

        let parent_failed = create_snapshot(TaskState::Failed, 1);
        assert_eq!(parent_failed.calculate_effective_progress(&children), 1.0);
    }

    fn create_snapshot(state: TaskState, weight: u64) -> TaskSnapshot {
        TaskSnapshot {
            id: uuid::Uuid::new_v4(),
            parent_id: None,
            name: "test".into(),
            state,
            progress: 0.0,
            weight,
            hidden_in_view: false,
            message: None,
        }
    }

    fn create_node(state: TaskState, progress: f64, weight: u64) -> TaskNode {
        TaskNode {
            id: uuid::Uuid::new_v4(),
            name: "node".into(),
            state,
            progress,
            message: None,
            weight,
            children: vec![],
        }
    }

    #[tokio::test]
    async fn race_context_should_allow_only_one_winner() {
        let race_ctx = RaceContext::new();
        assert!(race_ctx.try_win());
        assert!(!race_ctx.try_win());
    }

    #[tokio::test]
    async fn race_context_guard_should_rollback_on_drop() {
        let race_ctx = RaceContext::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        {
            let _guard = race_ctx.defer(move || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
        }

        sleep(Duration::from_millis(10)).await;

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn race_context_guard_should_not_rollback_if_committed() {
        let race_ctx = RaceContext::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        {
            let mut guard = race_ctx.defer(move || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            });
            guard.commit();
        }

        sleep(Duration::from_millis(10)).await;

        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn race_context_should_log_error_on_rollback_failure() {
        let race_ctx = RaceContext::new();

        {
            let _guard = race_ctx
                .defer(|| async { Err(anyhow!("Intentional rollback error for coverage")) });
        }

        sleep(Duration::from_millis(10)).await;
    }
}
