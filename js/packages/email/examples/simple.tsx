import { renderToString } from "../index";
import { Html, Head, Body, Container } from "../components";

const Email = (props: any) => {
  return (
    <Html>
      <Head></Head>
      <Body>
        <Container>
          <h1>Hello {props.name}</h1>
        </Container>
      </Body>
    </Html>
  );
};

const props = { name: "World" };
console.log(renderToString(<Email {...props} />));
