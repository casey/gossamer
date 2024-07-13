const response = await fetch("api/manifest");
const manifest = await response.json();

class App {
  constructor() {
    let html = '';

    if (manifest.media.type != "comic") {
      document.body.innerHTML = `<h1>app cannot handle content type \`${manifest.type}\``;
      return;
    }

    for (let i = 0; i < manifest.media.pages.length; i++) {
      if (i > 0) {
        html += '\n';
      }
      html += `<img src=content/${i}>`
    }

    document.body.innerHTML = html
  }
}

window.app = new App();
