import { appWindow } from '@tauri-apps/api/window';
import { createSignal, onMount, useContext } from 'solid-js';
import { LazyCircleProgress } from '~/lazy';
import { downloadExecutableFile, getExecutableFile, unzip } from '~/lib';
import notify from '~/lib/notify';
import { AppContext } from './context';

const Download = () => {
  const { download } = useContext(AppContext)!;

  const [percent, setPercent] = createSignal<number>(0);

  onMount(() => {
    onDownloading().then(() => download.setShow(false));
  });

  const updateProgress = (progress: number, total: number) => {
    const pgs = parseFloat(((progress / total) * 100).toFixed(1));

    setPercent(pgs);

    if (pgs === 100) {
      notify('核心组件下载完成');
    }
  };

  const onDownloading = async () => {
    notify('核心组件缺失，即将自动下载');

    setPercent(0);

    const unlisten = await appWindow.listen<DownloadProgress>(
      'download://progress',
      (e) => updateProgress(e.payload.progress, e.payload.total),
    );

    let filePath: string;

    try {
      filePath = await downloadExecutableFile();
      unlisten();
    } catch (e) {
      notify('核心组件下载失败，请检查网络状态');
      unlisten();

      return;
    }

    const executableFile = await getExecutableFile();

    const zip: Zip = {
      file_path: filePath,
      extract_files: [executableFile],
    };

    await unzip(zip);

    notify('核心组件下载并解压完成');
  };

  return (
    <div id="download">
      <LazyCircleProgress
        percent={percent()}
        size="large"
        text={{ undone: '正在下载核心模块...', done: '下载完成' }}
      />
    </div>
  );
};

export default Download;
