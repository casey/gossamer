const response = await fetch("api/manifest");
const manifest = await response.json();

class App {
  constructor() {
    let html = '';

    let pages = manifest['Comic'].pages.length;

    for (let i = 0; i < pages; i++) {
      html += `<img src=content/${i}>`
    }

    document.body.innerHTML = html
  }
}

window.app = new App();
