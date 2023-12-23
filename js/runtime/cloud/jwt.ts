namespace Arena {
  export let core: any;
}

type Header = {
  alg:
    | "HS256"
    | "HS384"
    | "HS512"
    | "RS256"
    | "RS384"
    | "RS512"
    | "PS256"
    | "PS384"
    | "PS512"
    | "ES256"
    | "ES384"
    | "EdDSA";
};

type JWT = {
  sign: (options: { header: Header; payload: any; secret: string }) => string;
  verify: (
    token: string,
    header: Header["alg"],
    secret: string
  ) => { header: Header; payload: any };
};

const sign = Arena.core.ops.op_cloud_jwt_sign as JWT["sign"];
const verify = Arena.core.ops.op_cloud_jwt_verify as JWT["verify"];

export { sign, verify };
