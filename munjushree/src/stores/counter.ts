import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useInfoStore = defineStore('info', () => {
  const name = ref("")
  function set_name(value: string) {
    name.value = value
  }

  return { name, set_name }
})
