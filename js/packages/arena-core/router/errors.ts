const error = (status: number, message: string) =>
  new Response(message, {
    status,
  });

const errors = {
  notFound(message?: string) {
    return error(404, message || "404 Not found");
  },
  badRequest(message?: string) {
    return error(400, message || "400 Bad request");
  },
  forbidden(message?: string) {
    return error(403, message || "403 Forbidden");
  },
  internalServerError(message?: string) {
    return error(500, message || "500 Internal Server Error");
  },
};

export { errors };
