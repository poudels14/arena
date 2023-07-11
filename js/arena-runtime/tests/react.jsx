import React from "react";

const Component = (props) => <div>Hello {props.name}</div>;

console.log(Component.toString());
console.log(<Component name={"world"} />);
