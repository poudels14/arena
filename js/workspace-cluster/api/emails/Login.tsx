import {
  Html,
  Head,
  Body,
  Container,
  Section,
  Img,
  Link,
  Hr,
} from "@portal/email/components";

const Login = (props: { host: string; magicLink: string }) => {
  return (
    <Html>
      <Head></Head>
      <Body>
        <Container>
          <Img
            src={`${props.host}/static/portal-logo.png`}
            width={48}
            height={48}
            alt="Sidecar"
          />
          <h1
            style={{
              "font-size": "28px",
              "font-weight": "bold",
              "margin-top": "48px",
            }}
          >
            🪄 Your magic link
          </h1>
          <Section style={body}>
            <p style={paragraph}>
              <Link style={link} href={props.magicLink}>
                👉 Click here to sign in 👈
              </Link>
            </p>
            <p style={paragraph}>
              If you didn't request this, please ignore this email.
            </p>
          </Section>
          <p style={paragraph}>
            Best,
            <br />
            Sidecar Team
          </p>
          <Hr style={hr} />
          <Img
            src={`${props.host}/static/portal-logo.png`}
            width={32}
            height={32}
            style={{
              WebkitFilter: "grayscale(100%)",
              filter: "grayscale(100%)",
              margin: "20px 0",
            }}
          />
          <p style={footer}>Sidecar Inc.</p>
        </Container>
      </Body>
    </Html>
  );
};

const body = {
  margin: "24px 0",
};

const paragraph = {
  "font-size": "16px",
  "line-height": "26px",
};

const link = {
  color: "#FF6363",
};

const hr = {
  "border-color": "#dddddd",
  "margin-top": "48px",
};

const footer = {
  color: "#8898aa",
  "font-size": "12px",
  "margin-left": "4px",
};

export { Login };
