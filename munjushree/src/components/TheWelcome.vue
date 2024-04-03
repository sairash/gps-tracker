<script setup lang="ts">
import L from 'leaflet';
import { Geolocation } from '@capacitor/geolocation';

// import {Peer} from "peerjs"

import { io, Socket } from "socket.io-client";
import { onMounted, ref } from 'vue';

interface LatLng {
  lat: number;
  lng: number;
  last_update: string
}

const map = ref<L.Map | null>(null);
let marker: L.Marker | null = null;
var layerGroup = L.layerGroup();

var markers: { [key: string]: LatLng; } = {};


function date(): string {
  const now = new Date();
  return now.toISOString();
}

function redraw_markers() {
  layerGroup.clearLayers();

  Object.keys(markers).forEach((key, index) => {
    if (markers[key] != null) {
      L.marker([markers[key].lat, markers[key].lng], {
        icon: L.icon({
        iconUrl: `https://ui-avatars.com/api/?rounded=true&name=${key}`,
        iconSize: [50, 50],
      }),
      }).bindPopup(`<b class="font-bold">Last Update: </b> ${markers[key].last_update}`).addTo(layerGroup);
    }
  });
}

onMounted(async () => {
  const coordinates = await Geolocation.getCurrentPosition();
  map.value = L.map('map').setView([coordinates.coords.latitude, coordinates.coords.longitude], 13);
  layerGroup.addTo(map.value as L.Map)
  // Add a tile layer
  L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
    attribution: '© OpenStreetMap contributors'
  }).addTo(map.value as L.Map);


  const socket = io("localhost:3000", {
    auth: {
      lat: coordinates.coords.latitude,
      lng: coordinates.coords.longitude,
      user_id: "1",
      room: "munjushree",
      last_update: date(),
    },
  });

  socket.on("location_update", (data) => {
    data = JSON.parse(data);
    if (Array.isArray(data)) {
      for (let i = 0; i < data.length; i++) {
        markers[data[i].id] = data[i].location
        redraw_markers();
      }
    } else {
      markers[data.id] = data.location
      redraw_markers();
    }
    console.log(data)
  });
});

</script>
<template>
  <div class="w-screen h-screen bg-yellow-500 flex flex-col px-2 py-4">
    <div class="bg-black w-full h-full rounded-xl " id="map"></div>
    <div class="w-full h-96 rounded-xl flex flex-col">
      <img src="../assets/walkietalkie.png" class="w-full px-20" alt="">
      <button
        class="h-36 w-full mb-3  bg-red-500 mt-2 rounded-xl text-xl font-bold grid items-center justify-center cursor-pointer main">
        Press To Speak
      </button>
    </div>
  </div>
  <!-- <input type="text" v-model="room" name="" id="">
  {{ room }} -->
</template>
