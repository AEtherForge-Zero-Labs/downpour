<script lang="ts">
  import { onMount } from 'svelte';
  import { native, dependencies, latestRelease, preview, chooseDirectory, cancel, reveal, download, type Options, type Release } from '#lib/platform.ts';

  let options = $state<Options>({ url: '', mode: 'video', format: 'mp4', quality: '1080', directory: '', playlist: false, subtitles: false, thumbnail: false, metadata: true, rateLimit: '', concurrentFragments: 4 });
  let ytDlp = $state<string | null>(null);
  let ffmpeg = $state(false);
  let loading = $state(true);
  let checking = $state(false);
  let release = $state<Release | null>(null);
  let updateError = $state('');
  let error = $state('');
  let busy = $state(false);
  let cancelling = $state(false);
  let status = $state('Ready when you are');
  let progress = $state(0);
  let logs = $state<string[]>([]);
  let advanced = $state(false);
  let command = $state('');
  let history = $state<{ title: string; format: string; folder: string; time: string }[]>([]);
  let tab = $state<'download' | 'history'>('download');
  let runningFormat = $state('');
  let runningFolder = $state('');
  const videoFormats = ['mp4', 'mkv', 'webm'];
  const audioFormats = ['mp3', 'm4a', 'wav', 'flac', 'opus'];
  const formats = $derived(options.mode === 'video' ? videoFormats : audioFormats);
  const ready = $derived(native && !loading && !!ytDlp && ffmpeg);

  onMount(() => {
    try { const saved = localStorage.getItem('downpour-settings'); if (saved) { const { url: _url, ...settings } = JSON.parse(saved); options = { ...options, ...settings, url: '' }; } } catch { /* A fresh start if storage is unavailable. */ }
    void refresh();
    if (native) void checkVersion();
  });
  async function refresh() {
    loading = true;
    if (!native) { loading = false; return; }
    try { const d = await dependencies(); ytDlp = d.ytDlp; ffmpeg = d.ffmpeg; if (!options.directory) options.directory = d.directory; }
    catch (e) { error = String(e); }
    finally { loading = false; }
  }
  function setMode(mode: 'video' | 'audio') {
    options.mode = mode; options.format = mode === 'video' ? 'mp4' : 'mp3'; options.quality = mode === 'video' ? '1080' : '0'; command = '';
  }
  async function checkVersion() {
    checking = true; updateError = '';
    try { release = await latestRelease(); }
    catch (e) { updateError = String(e); }
    finally { checking = false; }
  }
  async function folder() { try { const path = await chooseDirectory(); if (path) options.directory = path; } catch (e) { error = String(e); } }
  async function showFolder(path: string) { try { await reveal(path); } catch (e) { error = String(e); } }
  async function showCommand() { try { const args = await preview($state.snapshot(options)); command = ['yt-dlp', ...args].map(a => /^[\w./:=+-]+$/.test(a) ? a : JSON.stringify(a)).join(' '); error = ''; } catch (e) { error = String(e); } }
  async function start() {
    error = ''; logs = []; progress = 0; busy = true; cancelling = false; status = 'Connecting to YouTube…';
    const snapshot = $state.snapshot(options); runningFormat = snapshot.format; runningFolder = snapshot.directory;
    try {
      localStorage.setItem('downpour-settings', JSON.stringify({ ...snapshot, url: '' }));
    } catch { /* Downloading works without local storage. */ }
    try {
      await download(snapshot, (event) => {
        logs = [...logs.slice(-249), event.text];
        if (event.kind === 'log') {
          const match = event.text.match(/\[download\]\s+([\d.]+)%/);
          if (match) { progress = Number(match[1]); status = progress >= 100 ? 'Processing your files…' : 'Downloading…'; }
          if (/\[(Merger|ExtractAudio|VideoRemuxer|Metadata)\]/.test(event.text)) status = 'Processing your files…';
        } else {
          status = event.text;
          if (event.kind === 'complete') {
            progress = 100;
            history = [{ title: snapshot.url, format: snapshot.format, folder: snapshot.directory, time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }, ...history];
          }
          if (event.kind === 'error') error = event.text;
        }
      });
    } catch (e) { error = String(e); status = 'Download could not start'; }
    finally { busy = false; cancelling = false; }
  }
  async function stop() { cancelling = true; try { await cancel(); } catch (e) { error = String(e); cancelling = false; } }
</script>

<svelte:head><title>Downpour — Keep the good stuff.</title><meta name="description" content="A focused desktop interface for yt-dlp. Download YouTube video and audio in your preferred format." /></svelte:head>

<div class="app-shell">
  <aside class="sidebar">
    <a class="brand" href="./"><span class="brand-mark">↓</span> downpour<span class="brand-dot">.</span></a>
    <div class="workspace-label">YOUR WORKSPACE</div>
    <nav aria-label="Main navigation">
      <button class:active={tab === 'download'} onclick={() => tab = 'download'}><span>↧</span> New download <kbd>01</kbd></button>
      <button class:active={tab === 'history'} onclick={() => tab = 'history'}><span>◷</span> This session <span class="count">{history.length}</span></button>
    </nav>
    <div class="sidebar-bottom">
      <div class="engine-icon">⌘</div>
      <strong>Small app. Powerful engine.</strong>
      <p>All the capability of yt-dlp.<br />A little less command line.</p>
      <div class="local-badge"><span></span> Files stay on your device</div>
      <div class="app-version">DOWNPOUR <span>v0.1.0</span></div>
    </div>
  </aside>

  <main>
    <header><span class="breadcrumb">Workspace <span>/</span> {tab === 'download' ? 'New download' : 'This session'}</span><span class="desktop-pill">◉ DESKTOP APP</span></header>
    <div class="content">
      <div class="page-heading"><div class="eyebrow">LESS FRICTION. MORE OFFLINE.</div><h1>{tab === 'download' ? 'Keep the good stuff.' : 'Saved for later.'}</h1><p>{tab === 'download' ? 'Your favorite videos and audio, in the format you want.' : 'Downloads completed during this session.'}</p></div>
      {#if !native}<div class="notice"><strong>You’re viewing the browser preview.</strong> Downloads need the Downpour desktop app and a local installation of yt-dlp and FFmpeg. You can explore the options here.</div>{/if}
      {#if error}<div class="error" role="alert">{error}<button aria-label="Dismiss error" onclick={() => error = ''}>×</button></div>{/if}
      {#if tab === 'download'}
        <form onsubmit={(e) => { e.preventDefault(); if (ready && !busy) void start(); }}>
          <section class="card source-card"><div class="section-title"><span class="step">01</span><h2>Start with a link</h2><span class="section-note">YOUTUBE VIDEO OR PLAYLIST</span></div>
            <label class="sr-only" for="url">YouTube URL</label><div class="url-input"><span>↗</span><input id="url" type="url" placeholder="Paste a YouTube link here…" bind:value={options.url} disabled={busy} required /><span class="link-label">URL</span></div>
            <div class="source-footer"><span><i class="tiny-dot"></i> youtube.com & youtu.be</span><span>One link. All yours to keep.</span></div>
          </section>
          <section class="card"><div class="section-title"><span class="step">02</span><h2>Make it yours</h2></div>
            <fieldset disabled={busy}><legend class="sr-only">Download settings</legend>
              <div class="mode-tabs"><button type="button" class:selected={options.mode === 'video'} onclick={() => setMode('video')}><span>▣</span><div><strong>Video + audio</strong><small>The whole picture</small></div><span class="radio-dot"></span></button><button type="button" class:selected={options.mode === 'audio'} onclick={() => setMode('audio')}><span>♫</span><div><strong>Audio only</strong><small>Just the soundtrack</small></div><span class="radio-dot"></span></button></div>
              <div class="settings-grid"><div><label for="format">File format</label><select id="format" bind:value={options.format}>{#each formats as f}<option value={f}>{f.toUpperCase()}{f === 'mp4' || f === 'mp3' ? ' · Most compatible' : ''}</option>{/each}</select></div><div><label for="quality">{options.mode === 'video' ? 'Video quality' : 'Audio quality'}</label><select id="quality" bind:value={options.quality}>{#if options.mode === 'video'}<option value="best">Best available</option><option value="2160">2160p · 4K</option><option value="1440">1440p · QHD</option><option value="1080">1080p · Full HD</option><option value="720">720p · HD</option><option value="480">480p</option>{:else}<option value="0">Best available</option><option value="320K">320 kbps</option><option value="192K">192 kbps</option><option value="128K">128 kbps</option>{/if}</select></div></div>
              <p class="field-help">{options.mode === 'video' ? 'Downloads the best available quality up to your selection. FFmpeg merges and remuxes the file.' : 'FFmpeg converts the audio to your selected format. Lossless formats cannot restore quality lost at the source.'}</p>
              <div class="folder-label"><label for="directory">Save to</label><span>YOUR DEVICE, YOUR FILES</span></div><div class="folder-input"><span>▱</span><input id="directory" bind:value={options.directory} placeholder={native ? 'Choose an absolute folder path' : 'Your Downloads folder'} required /><button type="button" disabled={!native} onclick={folder}>Browse <span>…</span></button></div>
              <div class="options-list"><label><input type="checkbox" bind:checked={options.playlist} /><span>Download entire playlist<small>When the link includes a playlist</small></span></label><label><input type="checkbox" bind:checked={options.metadata} /><span>Embed metadata<small>Keep title, artist, and details</small></span></label></div>
              <button type="button" class="advanced-toggle" aria-expanded={advanced} onclick={() => advanced = !advanced}><span>⚙</span> Advanced options <span class="chevron">{advanced ? '−' : '+'}</span></button>
              {#if advanced}<div class="advanced-panel"><div class="options-list"><label><input type="checkbox" bind:checked={options.subtitles} /><span>Save English subtitles<small>Including auto-generated captions, as VTT</small></span></label><label><input type="checkbox" bind:checked={options.thumbnail} /><span>Save thumbnail<small>Alongside your downloaded file</small></span></label></div><div class="settings-grid"><div><label for="rate">Bandwidth limit</label><input id="rate" placeholder="Unlimited · e.g. 2M" bind:value={options.rateLimit} /></div><div><label for="fragments">Parallel fragments</label><input id="fragments" type="number" min="1" max="16" bind:value={options.concurrentFragments} /></div></div><button class="text-button" type="button" disabled={!native} onclick={showCommand}>Preview yt-dlp arguments ↗</button>{#if command}<pre class="command">{command}</pre>{/if}</div>{/if}
            </fieldset>
          </section>
          <div class="download-bar"><div><strong>{busy ? `Saving ${runningFormat.toUpperCase()}` : `Ready for ${options.format.toUpperCase()}`}</strong><span>{busy ? 'Keep this window open while downloading.' : 'Download only content you own or have permission to save.'}</span></div>{#if busy}<button class="cancel" type="button" disabled={cancelling} onclick={stop}>{cancelling ? 'Cancelling…' : 'Cancel download'}</button>{:else}<button class="primary" type="submit" disabled={!ready || !options.url.trim()}>Start download <span>↓</span></button>{/if}</div>
        </form>
        {#if logs.length || busy}<section class="card activity" aria-label="Download activity"><div class="activity-heading"><span class:spinning={busy}>◌</span><strong role="status">{status}</strong>{#if !busy && progress === 100}<button class="text-button" onclick={() => showFolder(runningFolder)}>Open folder ↗</button>{/if}</div><progress max="100" value={progress} aria-label="Current file progress"></progress><details open={!!error}><summary>Activity log <span>{logs.length} lines</span></summary><pre>{logs.join('\n')}</pre></details></section>{/if}
      {:else}<section class="card history">{#if !history.length}<div class="empty-icon">◷</div><h2>A little quiet here.</h2><p>Your completed downloads will appear here for this session.</p><button class="primary" onclick={() => tab = 'download'}>Make your first download ↓</button>{:else}{#each history as item}<div class="history-row"><span class="file-icon">{item.format.toUpperCase()}</span><div><strong>{item.title}</strong><small>Completed at {item.time}</small></div><button class="text-button" onclick={() => showFolder(item.folder)}>Open folder ↗</button></div>{/each}{/if}</section>{/if}
      <section class="engine-panel"><div class="engine-status"><span class:good={!!ytDlp && ffmpeg} class="status-light"></span><div><strong>{loading ? 'Checking your tools…' : native ? 'Powered by yt-dlp' : 'Powered by yt-dlp · Desktop required'}</strong><p>{native ? `yt-dlp ${ytDlp || 'not found'} · FFmpeg ${ffmpeg ? 'ready' : 'not found'}` : 'A familiar engine, a friendlier interface.'}</p></div></div><div class="engine-actions">{#if native}<button class="text-button" disabled={loading || busy} onclick={refresh}>Refresh tools</button><button class="version-button" disabled={checking} onclick={checkVersion}>{checking ? 'Checking…' : 'Check for updates'} <span>↻</span></button>{:else}<a href="https://github.com/yt-dlp/yt-dlp/releases/latest" target="_blank" rel="noreferrer">yt-dlp releases ↗</a>{/if}</div>
        {#if native && !loading && (!ytDlp || !ffmpeg)}<div class="setup-help"><strong>Two tools, one quick setup.</strong><p>Install yt-dlp and FFmpeg, then refresh. On macOS: <code>brew install yt-dlp ffmpeg</code>. On Windows: <code>winget install yt-dlp.yt-dlp</code> and <code>winget install Gyan.FFmpeg</code>. Linux: install both through your package manager. A supported JavaScript runtime such as Deno is also recommended for YouTube.</p><a href="https://github.com/yt-dlp/yt-dlp#installation" target="_blank" rel="noreferrer">Installation guide ↗</a></div>{/if}
        {#if release}<div class="release-info" role="status">{ytDlp === release.version ? `You're up to date: ${release.version}.` : `Latest stable: ${release.version}. Installed: ${ytDlp || 'not found'}.`} <a href={release.url} target="_blank" rel="noreferrer">Release & installation instructions ↗</a><p>Update yt-dlp through the same package manager or installer you used to install it, then refresh tools.</p></div>{/if}
        {#if updateError}<div class="release-info error" role="alert">{updateError}</div>{/if}
      </section>
      <footer><span>MADE FOR YOUR OFFLINE LIFE.</span><span>Built on open source. Saved locally.</span></footer>
    </div>
  </main>
</div>
