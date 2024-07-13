export function define(name, callback) {
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

    async connectedCallback() {
      console.log('connected');
      let shadow = this.attachShadow({ mode: 'closed' });
      let [root, connected] = await callback;
      shadow.appendChild(root.documentElement);
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
