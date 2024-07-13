export function define(name, callback) {
  class CustomElement extends HTMLElement {
    async connectedCallback() {
      let shadow = this.attachShadow({ mode: 'closed' });
      try {
        let [root, connected] = await callback;
        shadow.appendChild(root.documentElement);
        connected(shadow);
      } catch (error) {
        console.log(error);
      }
    }
  }

  customElements.define(name, CustomElement);
}
