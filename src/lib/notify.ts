import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/api/notification';
import type { Options } from '@tauri-apps/api/notification';

const notify = async (message: string | Options) => {
  let permissionGranted = await isPermissionGranted();
  if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === 'granted';
  }

  if (permissionGranted) {
    const options: Options =
      typeof message === 'string'
        ? { title: 'hunter', body: message, sound: 'default' }
        : {
          ...message,
          sound: 'default',
        };
    sendNotification(options);
  }
};

export default notify;
