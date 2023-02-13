console.log("-------------------------------------------");

const request = new Request("https://nodejs.org/api/");
console.log(request);
console.log("-------------------------------------------");

const response = new Response({ url: "https://nodejs.org/api/" });
console.log(response);
console.log("-------------------------------------------");
