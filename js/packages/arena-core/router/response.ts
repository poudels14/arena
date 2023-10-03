import { isDev } from "./env";

const generateResponse = (response: any, options?: { status?: number }) => {
  if (response instanceof Response) {
    return response;
  } else if (response instanceof Error) {
    if (isDev()) {
      return jsonResponse(500, {
        error: {
          cause: response.cause,
          stack: response.stack,
        },
      });
    }
    return new Response("500 Internal Server Error", {
      status: 500,
    });
  } else {
    const status = options?.status || 200;
    if (
      typeof response != "string" &&
      !(response instanceof Uint8Array) &&
      !(response instanceof Uint16Array)
    ) {
      return jsonResponse(status, response);
    }
    return new Response(response, {
      status,
    });
  }
};

const jsonResponse = (status: number, response: any) => {
  return new Response(JSON.stringify(response), {
    status,
    headers: new Headers([["content-type", "application/json"]]),
  });
};

export { generateResponse };
