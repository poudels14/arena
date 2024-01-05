interface Websocket extends AsyncIterator<any> {
  /**
   * returns 0 if sending message failed
   */
  send(data: any): Promise<number>;
  close(data?: any): Promise<void>;
  next(): Promise<any>;
}

type ServeConfig = {
  fetch: (req: Request) => Promise<Response>;
  websocket?: (websocket: Websocket, data: any) => void;
};

export const serve: (config: ServeConfig) => Promise<void>;
