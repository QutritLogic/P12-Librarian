<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Modal from './Modal.svelte';
  import icon from '../../src-tauri/icons/128x128@2x.png';
  import studioPhoto from '../assets/prophet12-studio.webp';

  const WEBSITE = 'https://qutrit.se/p12-librarian/';
  const PRODUCT_PAGE = 'https://sequential.com/product/prophet-12-keyboard/';
  const EMAIL = 'qutritlogic@proton.me';

  let { onClose }: { onClose: () => void } = $props();
  let version = $state('');

  onMount(() => {
    getVersion().then((v) => (version = v));
  });
</script>

<Modal title="About" width="480px" {onClose}>
  <div class="head">
    <img src={icon} alt="" width="72" height="72" />
    <div>
      <div class="name">P12 Librarian</div>
      <div class="muted">Patch librarian for the Sequential Prophet 12</div>
      <div class="muted">Version {version}</div>
      <div class="muted">Made by Qutrit and Claude (Anthropic)</div>
    </div>
  </div>

  <p>Back up, browse and restore programs on a Sequential Prophet 12 over MIDI SysEx.</p>

  <div class="synth">
    <figure>
      <img src={studioPhoto} alt="Close-up of the Prophet 12's knobs, lit red in a dark studio" width="960" height="320" />
      <figcaption>Prophet 12 in Qutrit's studio</figcaption>
    </figure>
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

  <p class="muted small">
    Contact:
    <button class="link" onclick={() => openUrl(`mailto:${EMAIL}`)} title={`Email ${EMAIL}`}>{EMAIL}</button>
  </p>
  <p class="muted small">Built with Rust, Tauri, Svelte and midir.</p>
  <p class="muted small">
    Themes after Dracula, Catppuccin, Tokyo Night, Gruvbox, One Dark Pro, Rosé Pine, Kanagawa, Everforest and
    Solarized (MIT). Fonts: Fira Code, JetBrains Mono and Hack Nerd Fonts (OFL / MIT). See Help → Licences.
  </p>
  <p class="muted small">
    Not affiliated with Sequential or Dave Smith Instruments. Prophet 12 is a trademark of its owner.
  </p>
  <p class="muted small">
    Free and open source under the MIT License; the source code is on GitHub. Provided as is, without
    warranty; use at your own risk. See Help → Licences and Help → Disclaimer.
  </p>

  {#snippet buttons()}
    <button class="link website" onclick={() => openUrl(WEBSITE)} title={WEBSITE}>Website</button>
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
  .synth {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  figure {
    margin: 0 0 4px;
  }
  figure img {
    display: block;
    width: 100%;
    height: auto;
    border-radius: 6px;
  }
  figcaption {
    margin-top: 4px;
    text-align: right;
    font-size: 11px;
    color: var(--muted);
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
  /* On the left of the buttons row, away from Close. */
  .website {
    margin-right: auto;
  }
  .muted {
    color: var(--muted);
  }
  .small {
    font-size: 12px;
  }
</style>
