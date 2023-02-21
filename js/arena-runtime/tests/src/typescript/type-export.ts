type NewType = {
  name: string;
  age: number;
};

const value: NewType = {
  name: "Test name",
  age: 25,
};

console.log(value);

export type { NewType };
export { value };
