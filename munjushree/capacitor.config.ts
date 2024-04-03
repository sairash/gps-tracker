import { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'com.munjushree.app',
  appName: 'Munjushree',
  webDir: 'dist',
  server: {
    androidScheme: 'https'
  }
};

export default config;
