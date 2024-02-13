import papaparse from "papaparse";

const convertToDataFrame = (data: any): any | null => {
  const parsed = papaparse.parse(data, {
    header: true,
    skipEmptyLines: true,
  });
  return {
    rows: parsed.data,
  };
};

export { convertToDataFrame };
