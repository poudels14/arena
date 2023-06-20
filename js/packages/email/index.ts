import { JSXElement } from "solid-js";
import { renderToString as solidRenderToString } from "solid-js/web";

const docType = `<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">`;

const renderToString = (Component: JSXElement) => {
  const markup = solidRenderToString(() => Component);
  return `${docType}${markup}`;
};

export { renderToString };
