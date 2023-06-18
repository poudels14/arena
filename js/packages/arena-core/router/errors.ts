const error = (status: number, message: string) =>
  new Response(message, {
    status,
  });

const errors = {
  notFound() {
    return error(404, "404 Not found");
  },
  badRequest() {
    return error(400, "400 Bad request");
  },
  forbidden() {
    return error(403, "403 Forbidden");
  },
  internalServerError() {
    return error(500, "500 Internal Server Error");
  },
};

export { errors };
