<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { listen } from '@tauri-apps/api/event';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Modal from './Modal.svelte';
  import icon from '../../src-tauri/icons/128x128@2x.png';
  import logoOnDark from '../assets/brand/lockup-on-dark.svg';
  import logoOnLight from '../assets/brand/lockup-on-light.svg';

  const WEBSITE = 'https://qutrit.se/p12-librarian/';
  const PRODUCT_PAGE = 'https://sequential.com/product/prophet-12-keyboard/';
  const EMAIL = 'qutritlogic@proton.me';

  let { onClose }: { onClose: () => void } = $props();
  let version = $state('');
  let darkTheme = $state(isDark());

  /** Whether the current theme is dark, judged from its surface colour. */
  function isDark(): boolean {
    const hex = getComputedStyle(document.documentElement).getPropertyValue('--surface').trim();
    const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
    return 0.299 * r + 0.587 * g + 0.114 * b < 128;
  }

  onMount(() => {
    getVersion().then((v) => (version = v));
    // The theme can change while the box is open; wait a frame for the new colours.
    const unlisten = listen('settings', () => requestAnimationFrame(() => (darkTheme = isDark())));
    return () => {
      unlisten.then((f) => f());
    };
  });
</script>

<Modal title="About" width="480px" {onClose}>
  <div class="head">
    <img src={icon} alt="" width="72" height="72" />
    <div>
      <div class="name">P12 Librarian</div>
      <div class="muted">Patch librarian for the Sequential Prophet 12</div>
      <div class="muted">Version {version}</div>
    </div>
  </div>

  <p>Back up, browse and restore programs on a Sequential Prophet 12 over MIDI SysEx.</p>

  <div class="credits">
    <p>
      <b>Created by Claude</b> (Opus 5.5), an AI model made by Anthropic, who wrote the code.
    </p>
    <img class="logo" src={darkTheme ? logoOnDark : logoOnLight} alt="Qutrit Logic" height="48" />
    <p>
      <b>Together with Qutrit</b>, who had the idea, made the design decisions and tested it on a real
      Prophet 12.
    </p>
    <p>
      Contact:
      <button class="link" onclick={() => openUrl(`mailto:${EMAIL}`)} title={`Email ${EMAIL}`}>{EMAIL}</button>
    </p>
  </div>

  <div class="synth">
    <p>
      The <b>Prophet 12</b> was designed by <b>Dave Smith Instruments (DSI)</b>, founded by Dave Smith, and released
      in 2013. The company is now called Sequential.
    </p>
    <p>
      <button
        class="link"
        onclick={() => openUrl(PRODUCT_PAGE)}
        title={PRODUCT_PAGE}>Prophet 12 on sequential.com ↗</button
      >
    </p>
  </div>

  <p class="muted small">Built with Rust, Tauri, Svelte and midir.</p>
  <p class="muted small">
    Themes after Dracula, Catppuccin, Tokyo Night, Gruvbox, One Dark Pro, Rosé Pine, Kanagawa, Everforest and
    Solarized (MIT). Fonts: Fira Code, JetBrains Mono and Hack Nerd Fonts (OFL / MIT). See Help → Licences.
  </p>
  <p class="muted small">
    Not affiliated with Sequential or Dave Smith Instruments. Prophet 12 is a trademark of its owner.
  </p>
  <p class="muted small">
    Freeware: free to use, not to redistribute or modify; get it from qutrit.se. Provided as is, without
    warranty; use at your own risk. See Help → Licences and Help → Disclaimer.
  </p>

  {#snippet buttons()}
    <button onclick={() => openUrl(WEBSITE)} title={WEBSITE}>Website: qutrit.se</button>
    <button class="primary" onclick={onClose} title="Close (Esc)">Close</button>
  {/snippet}
</Modal>

<style>
  .head {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .name {
    font-size: 18px;
    font-weight: 600;
  }
  p {
    margin: 0;
  }
  .credits {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
    border-radius: 6px;
    background: var(--surface-2);
  }
  .logo {
    align-self: flex-start;
    height: 48px;
    width: auto;
    margin: 4px 0 -2px -8px;
  }
  .synth {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .link {
    border: none;
    background: none;
    padding: 0;
    color: var(--accent);
    text-decoration: underline;
  }
  .link:hover:not(:disabled) {
    background: none;
    filter: brightness(1.2);
  }
  .muted {
    color: var(--muted);
  }
  .small {
    font-size: 12px;
  }
</style>
