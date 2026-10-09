import { mount } from 'svelte';
import App from './App.svelte';
import '@fontsource/fira-code/400.css';
import '@fontsource/fira-code/600.css';
import './app.css';

export default mount(App, { target: document.getElementById('app')! });
