<template>
  <q-layout view='hHh Lpr lFf'>
    <q-header>
      <q-toolbar class='text-white bg-white vertical-menu-padding shadow-2'>
        <Header class='full-width' />
      </q-toolbar>
    </q-header>
    <q-page-container>
      <router-view />
    </q-page-container>
  </q-layout>
</template>

<script setup lang='ts'>
import { onMounted } from 'vue'
import { notify } from 'src/localstore'
import initWasm from '../../dist/wasm/linera_wasm'
import wasmModuleUrl from '../../dist/wasm/linera_wasm_bg.wasm?url'

import Header from 'src/components/header/Header.vue'

const _notify = notify.useNotificationStore()

onMounted(async () => {
  await initWasm(await fetch(wasmModuleUrl))

  _notify.$subscribe((_, state) => {
    state.Notifications.forEach((notif, index) => {
      if (notif.Popup) {
        state.Notifications.splice(index, 1)
        notify.notify(notif)
      }
    })
  })
})

</script>
