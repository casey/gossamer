export function define(name, callback) {
  class CustomElement extends HTMLElement {
    async connectedCallback() {
      let shadow = this.attachShadow({ mode: 'closed' });
      let [root, connected] = await callback;
      shadow.appendChild(root.documentElement);
      connected(shadow);
    }
  }

  customElements.define(name, CustomElement);
}
