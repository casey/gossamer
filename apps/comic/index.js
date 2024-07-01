const response = await fetch("api/manifest");
const manifest = await response.json();

class App {
  constructor() {
    let html = '';

    for (let i = 0; i < manifest.pages.length; i++) {
      if (i > 0) {
        html += '\n';
      }
      html += `<img src=content/${i}>`
    }

    document.body.innerHTML = html
  }
}

window.app = new App();
