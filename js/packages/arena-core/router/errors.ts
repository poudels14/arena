import { generateResponse } from "./response";

const error = (status: number, message: any) => {
  return generateResponse(message, {
    status,
  });
};

const errors = {
  notFound(message?: any) {
    return error(404, message || "404 Not found");
  },
  badRequest(message?: any) {
    return error(400, message || "400 Bad request");
  },
  forbidden(message?: any) {
    return error(403, message || "403 Forbidden");
  },
  internalServerError(message?: any) {
    return error(500, message || "500 Internal Server Error");
  },
};

export { errors };
