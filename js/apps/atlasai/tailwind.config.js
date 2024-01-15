const solidUi = require("@portal/solid-ui/tailwind");

/** @type {import('tailwindcss').Config} */
module.exports = {
  theme: {
    extend: {
      colors: {
        "brand-12": "red" 
      },
    },
  },
  plugins: [solidUi],
};
