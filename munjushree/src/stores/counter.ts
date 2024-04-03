import { ref } from 'vue'
import { defineStore } from 'pinia'
import { Preferences } from '@capacitor/preferences';



export const useInfoStore = defineStore('info', () => {
  async function set_name(value: string) {
    await Preferences.set({
      key: 'name',
      value: value
    });
  }

  async function set_color(value: string) {
    await Preferences.set({
      key: 'color',
      value: value
    });
  }

  async function get_name():Promise<string> {
    const name = (await Preferences.get({
      key: 'name',
    })).value as string;

    return name == null? "": name;
  }

  async function get_color():Promise<number> {
    const name = parseInt((await Preferences.get({
      key: 'color',
    })).value as string);

    return Number.isNaN(name)?0:name;
  }



  return { set_name, set_color, get_name, get_color }
})
