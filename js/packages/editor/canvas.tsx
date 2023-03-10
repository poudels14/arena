const GridLines = (props: { size: number }) => {
  return (
    <div class="absolute w-full h-full bg-[length:40px_30px] bg-[linear-gradient(to_right,transparent,transparent,99%,rgba(51,65,85,0.2)),linear-gradient(to_top,transparent,transparent,99%,rgba(51,65,85,0.2))]"></div>
  );
};

const Canvas = () => {
  return (
    <div class="relative w-full h-full">
      <GridLines size={50} />
    </div>
  );
};

export { Canvas };
