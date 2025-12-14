use crate::scheduler::model::TaskSnapshot;
use crate::scheduler::{Scheduler, TaskNode};
use std::collections::HashMap;
use uuid::Uuid;

impl Scheduler {
    pub fn tree_view(&self) -> Vec<TaskNode> {
        let mut children_map: HashMap<Option<Uuid>, Vec<TaskSnapshot>> = HashMap::new();

        self.registry
            .iter()
            .map(|r| r.value().clone())
            .filter(|snapshot| !snapshot.hidden_in_view)
            .for_each(|snap| {
                children_map.entry(snap.parent_id).or_default().push(snap);
            });

        Self::build_nodes(None, &children_map)
    }

    pub fn get_snapshot(&self, id: Uuid) -> Option<TaskSnapshot> {
        self.registry.get(&id).map(|entry| entry.value().clone())
    }

    fn build_nodes(
        parent_id: Option<Uuid>,
        map: &HashMap<Option<Uuid>, Vec<TaskSnapshot>>,
    ) -> Vec<TaskNode> {
        if let Some(children) = map.get(&parent_id) {
            let mut nodes = Vec::new();

            for child_snap in children {
                let child_nodes = Self::build_nodes(Some(child_snap.id), map);
                let child_progress = child_snap.calculate_effective_progress(&child_nodes);

                nodes.push(TaskNode {
                    id: child_snap.id,
                    name: child_snap.name.clone(),
                    state: child_snap.state,
                    progress: child_progress,
                    message: child_snap.message.clone(),
                    weight: child_snap.weight,
                    children: child_nodes,
                });
            }

            nodes
        } else {
            Vec::new()
        }
    }
}
