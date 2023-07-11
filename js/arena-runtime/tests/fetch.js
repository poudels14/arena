let url = null;
url = "http://localhost:8000";
url = "http://google.com";

fetch(url)
  .then((x) => x.text())
  .then((x) => console.log(x));
