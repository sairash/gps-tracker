<script setup lang="ts">
import L from 'leaflet';
import { showNotify } from 'vant';
import { Geolocation, type Position } from '@capacitor/geolocation';
import { useInfoStore } from '@/stores/counter';
import Hammer from 'hammerjs';

import {Peer} from "peerjs"

import { io, Socket } from "socket.io-client";
import { onMounted, ref } from 'vue';
import { showLoadingToast, closeToast } from 'vant';
import { watch } from 'vue';



const prop = defineProps<{
  is_admin: boolean
}>()


interface LatLng {
  lat: number;
  lng: number;
  last_update: string,
  selected_color: number,
}


const calling_person = ref("")
var buttonHolding = false;

var active = ref(false)
var button_value = ref("Press To Hangup")

const info = useInfoStore()
const buttonElement = ref<HTMLElement | null>(null);
const audioElement = ref<HTMLAudioElement | null>(null);
var local_stream:MediaStream;



const socketio = ref<Socket>()
const peer = ref<Peer>()

const map = ref<L.Map | null>(null);
const anchors = [
  0,
  250,
];

const selected_color = ref(0)

const colors = [
  "ef4444",
  "f97316",
  "f97316",
  "f59e0b",
  "eab308",
  "84cc16",
  "22c55e",
  "10b981",
  "14b8a6",
  "06b6d4",
  "0ea5e9",
  "3b82f6",
  "6366f1",
  "8b5cf6",
  "a855f7",
  "d946ef",
  "ec4899",
  "f43f5e",
  "64748b",
  "6b7280",
  "71717a",
  "737373",
  "78716c",
];

const auto_text = [
  1,
  1,
  1,
  1,
  1,
  1,
  1,
  1,
  1,
  1,
  1,
  0,
  0,
  0,
  0,
  1,
  1,
  1,
  0,
  0,
  0,
  0,
  0,
];

const color_text = [
  "fff",
  "000"
]


const height = ref(anchors[0]);
const username = ref("");

const watch_location_id = ref("");


const onSubmit = async (values: any) => {
  info.set_name(values.Username as string)
  info.set_color(`${selected_color.value}`)
  height.value = anchors[0];
  await start_socket();
};


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
          iconUrl: `https://ui-avatars.com/api/?rounded=true&name=${key}&background=${colors[markers[key].selected_color]}&color=${color_text[auto_text[markers[key].selected_color]]}`,
          iconSize: [50, 50],
        }),
      }).bindPopup(`<b class="font-bold">Last Update: </b> ${markers[key].last_update}`).addTo(layerGroup);
    }
  });
}


function socket_call_state(user_id: string, state: string){
  socketio.value?.emit("call", {
    id: user_id,
    state: state
  })
}

async function location_watch() {
  watch_location_id.value = await Geolocation.watchPosition({
    enableHighAccuracy: true,
    timeout: 5000,
    maximumAge: 0,
  }, location_update)
}


// async function clearWatch() {
//   if (watch_location_id.value != "") {
//     socketio.value?.disconnect()
//     await Geolocation.clearWatch({ id: watch_location_id.value })
//     watch_location_id.value = "";
//   }
// }


async function start_socket() {
  // await clearWatch();
  const coordinates = await Geolocation.getCurrentPosition();
  socketio.value = io("https://gps.sairashgautam.com.np/", {
    // socketio.value = io("localhost:3000", {
    auth: {
      lat: coordinates.coords.latitude,
      lng: coordinates.coords.longitude,
      user_id: await info.get_name(prop.is_admin),
      selected_color: await info.get_color(prop.is_admin),
      room: "munjushree",
      last_update: date(),
    },
  });

  socketio.value.on("location_update", (data) => {
    data = JSON.parse(data);
    if (Array.isArray(data)) {
      for (let i = 0; i < data.length; i++) {
        markers[data[i].id] = data[i].location
        markers[data[i].id]["selected_color"] = data[i].selected_color;
        redraw_markers();
      }
    } else {
      markers[data.id] = data.location
      markers[data.id]["selected_color"] = data.selected_color;
      redraw_markers();
    }
  });

  socketio.value.on("call", (data) => {
    data = JSON.parse(data);
    if(data.state == "call" && username.value == "1 Admin"){
      if(buttonHolding || calling_person.value != ""){
        return socket_call_state(data.id, "exit")
      }
      calling_person.value = data.id;
    }else if (data.state == "start" && username.value != "1 Admin"){
      if(!buttonHolding){
        return socket_call_state(data.id, "exit")
      }
      calling_person.value = data.id;
      // Webrtc 
      peer_join();
      // Join Room
    }else if(data.state == "exit" && calling_person.value == data.id){
      buttonHolding = false;
      calling_person.value = ""
      showNotify({
        message: `${data.id} stopped calling!`,
        className: "on-top",
      });
      console.log("Reloading!")
      window.location.reload();
    }
    else{
      buttonHolding = false;
      calling_person.value = ""
      showNotify({
        message: 'Busy or Disconnected!',
        className: "on-top",
      });
    }
    console.log(calling_person.value)
    console.log("Call ->", data)
  });

  // setTimeout(async() => {
  //   await location_watch()
  // }, 5000)
}


async function peer_join() {
  if (peer.value == undefined){
    peer.value = new Peer(username.value, {
        port: 443,
        path: '/'
    });
  }

  peer.value?.on('open', (id) => {
    console.log("Peer Room ID: ", id)
    navigator.mediaDevices.getUserMedia({ video: false, audio: true })
    .then(stream => {
        if(peer.value != undefined){
          let call = peer.value.call("1 Admin", stream)
          call.on('stream', (stream) => {
              setRemoteStream(stream);
          })
        }
    })
  })
}

function setRemoteStream(stream:MediaStream) {
  if(audioElement.value){
    audioElement.value.srcObject = stream
    audioElement.value.volume = 1;
    audioElement.value.play();
  }
}

async function peer_start() {
  if (peer.value == undefined){
    peer.value = new Peer(username.value, {
        port: 443,
        path: '/'
    });
  }
  peer.value?.on('open', (id) => {

        socket_call_state(calling_person.value, "start")

        navigator.mediaDevices.getUserMedia({ video: false, audio: true })
        .then(stream => {
            local_stream = stream;
        })
    peer.value?.on('call', (call) => {
        call.answer(local_stream);
        call.on('stream', (stream) => {
            setRemoteStream(stream)
        })
    })
  })
}

async function peer_end() {
  if (peer.value != undefined){
    peer.value.disconnect();
    peer.value.destroy();
  }
}


onMounted(async () => {
  const medi = await navigator.mediaDevices.getUserMedia({ audio: true })
  medi.getTracks().forEach(track => {
    track.stop();
  });

  username.value = await info.get_name(prop.is_admin);
  if(username.value == "1 Admin"){
    button_value.value = "Press to Receive!"
  }else{
    button_value.value = "Press to Speak!"
  }
  selected_color.value = await info.get_color(prop.is_admin);

  const coordinates = await Geolocation.getCurrentPosition();
  map.value = L.map('map', { zoomControl: true, zoom: 1, zoomAnimation: false, fadeAnimation: true, markerZoomAnimation: true }).setView([coordinates.coords.latitude, coordinates.coords.longitude], 13);
  layerGroup.addTo(map.value as L.Map)
  // Add a tile layer
  L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
    attribution: '© OpenStreetMap contributors'
  }).addTo(map.value as L.Map);


  if (username.value == "") {
    height.value = anchors[1]
  } else {
    await start_socket();
  }

  setInterval( async ()=>{
    await get_pos()
  }, 10000)
  
});




async function press_location() {
  showLoadingToast({
    message: 'Getting \n Location',
    forbidClick: true,
  });
  await get_pos()
}

async function get_pos(){
  const position = await Geolocation.getCurrentPosition();
  closeToast();
  location_update(position, null)
}

async function location_update(position: Position | null, err?: any) {
  socketio.value?.emit("location_update", {
    lat: position?.coords.latitude,
    lng: position?.coords.longitude,
    last_update: date(),
  })
}


function open_settings() {
  height.value = anchors[1]
}

function change_color(key: number) {
  selected_color.value = key
}

function clickToClose() {
  if (username.value != '') {
    height.value = anchors[0];
  } else {
    showNotify({
      message: 'Fill in Username First!',
      className: "on-top",
    });
  }
}

function active_or_inactive() {
  console.log("Here")
  if(!active.value){
    buttonElement.value?.classList.add('main-press');

    buttonHolding = true;
      if(username.value != "1 Admin"){
        button_value.value = "Press to Hangup!"
        showNotify({
          message: 'Wait for Admin To Receive',
          className: "on-top",
        });
        socket_call_state("1 Admin", "call")
      }else if(username.value == "1 Admin" && calling_person.value != ""){
        button_value.value = "Press to Hangup!"
        peer_start()
      }
    active.value = true
  }else{
    buttonElement.value?.classList.remove('main-press');
    if(username.value != "1 Admin"){
        button_value.value = "Press to Speak!"
        socket_call_state("1 Admin", "exit")
      }else{
        button_value.value = "Press to Receive!"
        socket_call_state(calling_person.value, "exit")
      }

      calling_person.value = ""
      buttonHolding = false;
      setTimeout(()=>{
        window.location.reload();
      }, 1000)
      peer_end();
      active.value = false
  }
}

</script>
<template>
  <div class="w-screen h-screen bg-yellow-500 flex flex-col px-2 py-4">
    <button @click="press_location()" class="absolute right-5 bottom-52 h-10 w-10 bg-green-500 p-2 rounded-lg location">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-full h-full color-white" viewBox="0 0 24 24" fill="none">
        <path
          d="M19 12C19 15.866 15.866 19 12 19C8.13401 19 5 15.866 5 12C5 8.13401 8.13401 5 12 5C15.866 5 19 8.13401 19 12Z"
          stroke="white" stroke-width="2" />
        <path d="M19 12H21" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M3 12H5" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M12 19L12 21" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M12 3L12 5" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        <path
          d="M15 12C15 13.6569 13.6569 15 12 15C10.3431 15 9 13.6569 9 12C9 10.3431 10.3431 9 12 9C13.6569 9 15 10.3431 15 12Z"
          stroke="white" stroke-width="2" />
      </svg>

    </button>

    <button @click="open_settings()" class="absolute left-5 bottom-52 h-10 w-10 bg-green-500 p-2 rounded-lg settings">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-full h-full color-white" viewBox="0 0 24 24" fill="none">
        <g id="Interface / Settings">
          <g id="Vector">
            <path
              d="M20.3499 8.92293L19.9837 8.7192C19.9269 8.68756 19.8989 8.67169 19.8714 8.65524C19.5983 8.49165 19.3682 8.26564 19.2002 7.99523C19.1833 7.96802 19.1674 7.93949 19.1348 7.8831C19.1023 7.82677 19.0858 7.79823 19.0706 7.76998C18.92 7.48866 18.8385 7.17515 18.8336 6.85606C18.8331 6.82398 18.8332 6.79121 18.8343 6.72604L18.8415 6.30078C18.8529 5.62025 18.8587 5.27894 18.763 4.97262C18.6781 4.70053 18.536 4.44993 18.3462 4.23725C18.1317 3.99685 17.8347 3.82534 17.2402 3.48276L16.7464 3.1982C16.1536 2.85658 15.8571 2.68571 15.5423 2.62057C15.2639 2.56294 14.9765 2.56561 14.6991 2.62789C14.3859 2.69819 14.0931 2.87351 13.5079 3.22396L13.5045 3.22555L13.1507 3.43741C13.0948 3.47091 13.0665 3.48779 13.0384 3.50338C12.7601 3.6581 12.4495 3.74365 12.1312 3.75387C12.0992 3.7549 12.0665 3.7549 12.0013 3.7549C11.9365 3.7549 11.9024 3.7549 11.8704 3.75387C11.5515 3.74361 11.2402 3.65759 10.9615 3.50224C10.9334 3.48658 10.9056 3.46956 10.8496 3.4359L10.4935 3.22213C9.90422 2.86836 9.60915 2.69121 9.29427 2.62057C9.0157 2.55807 8.72737 2.55634 8.44791 2.61471C8.13236 2.68062 7.83577 2.85276 7.24258 3.19703L7.23994 3.1982L6.75228 3.48124L6.74688 3.48454C6.15904 3.82572 5.86441 3.99672 5.6517 4.23614C5.46294 4.4486 5.32185 4.69881 5.2374 4.97018C5.14194 5.27691 5.14703 5.61896 5.15853 6.3027L5.16568 6.72736C5.16676 6.79166 5.16864 6.82362 5.16817 6.85525C5.16343 7.17499 5.08086 7.48914 4.92974 7.77096C4.9148 7.79883 4.8987 7.8267 4.86654 7.88237C4.83436 7.93809 4.81877 7.96579 4.80209 7.99268C4.63336 8.26452 4.40214 8.49186 4.12733 8.65572C4.10015 8.67193 4.0715 8.68752 4.01521 8.71871L3.65365 8.91908C3.05208 9.25245 2.75137 9.41928 2.53256 9.65669C2.33898 9.86672 2.19275 10.1158 2.10349 10.3872C2.00259 10.6939 2.00267 11.0378 2.00424 11.7255L2.00551 12.2877C2.00706 12.9708 2.00919 13.3122 2.11032 13.6168C2.19979 13.8863 2.34495 14.134 2.53744 14.3427C2.75502 14.5787 3.05274 14.7445 3.64974 15.0766L4.00808 15.276C4.06907 15.3099 4.09976 15.3266 4.12917 15.3444C4.40148 15.5083 4.63089 15.735 4.79818 16.0053C4.81625 16.0345 4.8336 16.0648 4.8683 16.1255C4.90256 16.1853 4.92009 16.2152 4.93594 16.2452C5.08261 16.5229 5.16114 16.8315 5.16649 17.1455C5.16707 17.1794 5.16658 17.2137 5.16541 17.2827L5.15853 17.6902C5.14695 18.3763 5.1419 18.7197 5.23792 19.0273C5.32287 19.2994 5.46484 19.55 5.65463 19.7627C5.86915 20.0031 6.16655 20.1745 6.76107 20.5171L7.25478 20.8015C7.84763 21.1432 8.14395 21.3138 8.45869 21.379C8.73714 21.4366 9.02464 21.4344 9.30209 21.3721C9.61567 21.3017 9.90948 21.1258 10.4964 20.7743L10.8502 20.5625C10.9062 20.5289 10.9346 20.5121 10.9626 20.4965C11.2409 20.3418 11.5512 20.2558 11.8695 20.2456C11.9015 20.2446 11.9342 20.2446 11.9994 20.2446C12.0648 20.2446 12.0974 20.2446 12.1295 20.2456C12.4484 20.2559 12.7607 20.3422 13.0394 20.4975C13.0639 20.5112 13.0885 20.526 13.1316 20.5519L13.5078 20.7777C14.0971 21.1315 14.3916 21.3081 14.7065 21.3788C14.985 21.4413 15.2736 21.4438 15.5531 21.3855C15.8685 21.3196 16.1657 21.1471 16.7586 20.803L17.2536 20.5157C17.8418 20.1743 18.1367 20.0031 18.3495 19.7636C18.5383 19.5512 18.6796 19.3011 18.764 19.0297C18.8588 18.7252 18.8531 18.3858 18.8417 17.7119L18.8343 17.2724C18.8332 17.2081 18.8331 17.1761 18.8336 17.1445C18.8383 16.8247 18.9195 16.5104 19.0706 16.2286C19.0856 16.2007 19.1018 16.1726 19.1338 16.1171C19.166 16.0615 19.1827 16.0337 19.1994 16.0068C19.3681 15.7349 19.5995 15.5074 19.8744 15.3435C19.9012 15.3275 19.9289 15.3122 19.9838 15.2818L19.9857 15.2809L20.3472 15.0805C20.9488 14.7472 21.2501 14.5801 21.4689 14.3427C21.6625 14.1327 21.8085 13.8839 21.8978 13.6126C21.9981 13.3077 21.9973 12.9658 21.9958 12.2861L21.9945 11.7119C21.9929 11.0287 21.9921 10.6874 21.891 10.3828C21.8015 10.1133 21.6555 9.86561 21.463 9.65685C21.2457 9.42111 20.9475 9.25526 20.3517 8.92378L20.3499 8.92293Z"
              stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
            <path
              d="M8.00033 12C8.00033 14.2091 9.79119 16 12.0003 16C14.2095 16 16.0003 14.2091 16.0003 12C16.0003 9.79082 14.2095 7.99996 12.0003 7.99996C9.79119 7.99996 8.00033 9.79082 8.00033 12Z"
              stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
          </g>
        </g>
      </svg>

    </button>
    <div class="bg-white w-full h-full rounded-xl " id="map"></div>
    <div class="w-full h-96 rounded-xl flex flex-col">
      <img src="../assets/walkietalkie.png" class="w-full px-20" alt="">
      <button
        class="h-36 w-full mb-3 bg-red-500 mt-2 rounded-xl text-xl font-bold grid items-center justify-center cursor-pointer main"
        v-if="username != '1 Admin'" ref="buttonElement" @click="active_or_inactive()" >
        {{button_value}}
      </button>
      <div v-else class="h-36 w-full mb-3 mt-2">
        <button
        class="h-36 w-full mb-3 bg-red-500 mt-2 rounded-xl text-xl font-bold grid items-center justify-center cursor-pointer main"
        v-if="username == '1 Admin' && calling_person != ''" ref="buttonElement" @click="active_or_inactive()" >
        {{button_value}}
      </button>
      </div>
    </div>
  </div>
  <van-floating-panel v-model:height="height" :anchors="anchors" style="z-index: 10000000; ">
    <van-form @submit="onSubmit">
      <van-cell-group inset>
        <van-field v-model="username" name="Username" label="Username" placeholder="Username"
          :rules="[{ required: true, message: 'Username is required' }]" />
        <div class="text-sm pl-3.5 mt-1">Colour: </div>
        <div class="h-16 flex overflow-y-auto pt-3 pl-1">
          <div @click="change_color(key)" class="w-10 h-10 rounded-xl mx-2" v-for="color, key in colors"
            style="flex-shrink: 0"
            :style="{ backgroundColor: `#${color}`, border: selected_color == key ? '2px solid black' : '' }">
          </div>
        </div>
      </van-cell-group>
      <div style="margin: 16px;">
        <van-button round block class="bg-red-500 border-0 font-bold" style="color: white !important;"
          native-type="submit">
          Walkie Talkie
        </van-button>
      </div>
    </van-form>
  </van-floating-panel>
  <van-overlay :show="height != anchors[0]" @click="clickToClose()"  style="z-index: 100000;">
  </van-overlay>

  <audio ref="audioElement" autoplay class="hidden"></audio>
  <!-- <input type="text" v-model="room" name="" id="">
  {{ room }} -->
</template>
