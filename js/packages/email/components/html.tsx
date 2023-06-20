const Html = (props: any) => {
  return (
    <>
      <meta http-equiv="Content-Type" content="text/html charset=UTF-8" />
      <html lang="en">{props.children}</html>
    </>
  );
};

export { Html };
