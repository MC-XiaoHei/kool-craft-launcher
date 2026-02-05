import { computed, ref, watch } from "vue"
import Ajv, { ValidateFunction } from "ajv"
import type { JSONSchema7 } from "json-schema"

const ajv = new Ajv({ allErrors: true })

export function useSchemaValidation(
  schema: JSONSchema7,
  modelValue: any,
  additionValidator?: (val: any) => string | null | Promise<string | null>,
) {
  const errorMsg = ref("")
  const validateSchema = computed<ValidateFunction>(() => {
    try {
      return ajv.compile(schema)
    } catch (e) {
      console.error("Invalid Schema Compilation:", e)
      throw e
    }
  })

  const validate = async (val: any) => {
    errorMsg.value = ""

    const valid = validateSchema.value(val)
    if (!valid && validateSchema.value.errors) {
      errorMsg.value = validateSchema.value.errors[0].message || "Invalid value"
      return false
    }

    if (additionValidator) {
      const customErr = await additionValidator(val)
      if (customErr) {
        errorMsg.value = customErr
        return false
      }
    }

    return true
  }

  watch(
    () => modelValue.value,
    async newVal => {
      await validate(newVal)
    },
  )

  return {
    errorMsg,
    validate,
  }
}
