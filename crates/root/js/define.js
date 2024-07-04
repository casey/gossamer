export function define(name, html, connected) {
  class CustomElement extends HTMLElement {
    static get observedAttributes() {
    }

    constructor() {
      console.log('constructor');
      super();
    }

    attributeChangedCallback(name, stale, fresh) {
      console.log('attribute', name, stale, fresh);
    }

    connectedCallback() {
      console.log('connected');
      let shadow = this.attachShadow({ mode: 'closed' });
      shadow.appendChild(html.documentElement);
      connected(shadow);
    }

    disconnectedCallback() {
      console.log('disconnected');
    }

    adoptedCallback() {
      console.log('adopted');
    }
  }

  customElements.define(name, CustomElement);
}
