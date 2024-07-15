export function define(name, callback) {
  class CustomElement extends HTMLElement {
    async connectedCallback() {
      await callback(this);
    }
  }

  customElements.define(name, CustomElement);
}
