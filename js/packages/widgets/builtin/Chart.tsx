import { default as uPlot } from "uplot";
import { onMount } from "solid-js";
import type { Template } from "../";

const metadata: Template.Metadata<{}> = {
  id: "at-chart",
  name: "Chart",
  description: "Chart",
  data: {},
  class: "bg-white",
};

const Chart = (props: Template.Props<{}>) => {
  let ref: any;
  onMount(() => {
    let data = [
      [1546300800, 1546387200],
      [35, 71],
      [90, 15],
    ];

    let opts = {
      title: "My Chart",
      id: "chart1",
      class: "my-chart",
      width: 400,
      height: 300,
      series: [
        {},
        {
          show: true,
          spanGaps: false,
          label: "RAM",
          value: (self: any, rawValue: any) => {
            return "$" + rawValue?.toFixed(2);
          },
          stroke: "rgba(240,0,0,0.9)",
          width: 1,
          fill: "rgba(255, 0, 0, 0.2)",
          dash: [10, 5],
        },
      ],
    };

    let uplot = new uPlot(opts, data as any, ref);
  });
  return (
    <>
      <style>
        {`.uplot, .uplot *, .uplot *::before, .uplot *::after {box-sizing: border-box;}.uplot {line-height: 1.5;width: min-content;}.u-title {text-align: center;font-size: 18px;font-weight: bold;}.u-wrap {position: relative;user-select: none;}.u-over, .u-under {position: absolute;}.u-under {overflow: hidden;}.uplot canvas {display: block;position: relative;width: 100%;height: 100%;}.u-axis {position: absolute;}.u-legend {font-size: 14px;margin: auto;text-align: center;}.u-inline {display: block;}.u-inline * {display: inline-block;}.u-inline tr {margin-right: 16px;}.u-legend th {font-weight: 600;}.u-legend th > * {vertical-align: middle;display: inline-block;}.u-legend .u-marker {width: 1em;height: 1em;margin-right: 4px;background-clip: padding-box !important;}.u-inline.u-live th::after {content: ":";vertical-align: middle;}.u-inline:not(.u-live) .u-value {display: none;}.u-series > * {padding: 4px;}.u-series th {cursor: pointer;}.u-legend .u-off > * {opacity: 0.3;}.u-select {background: rgba(0,0,0,0.07);position: absolute;pointer-events: none;}.u-cursor-x, .u-cursor-y {position: absolute;left: 0;top: 0;pointer-events: none;will-change: transform;z-index: 100;}.u-hz .u-cursor-x, .u-vt .u-cursor-y {height: 100%;border-right: 1px dashed #607D8B;}.u-hz .u-cursor-y, .u-vt .u-cursor-x {width: 100%;border-bottom: 1px dashed #607D8B;}.u-cursor-pt {position: absolute;top: 0;left: 0;border-radius: 50%;border: 0 solid;pointer-events: none;will-change: transform;z-index: 100;/*this has to be !important since we set inline "background" shorthand */background-clip: padding-box !important;}.u-axis.u-off, .u-select.u-off, .u-cursor-x.u-off, .u-cursor-y.u-off, .u-cursor-pt.u-off {display: none;}`}
      </style>
      <div ref={ref} {...props.attributes}></div>
    </>
  );
};

export default Chart;
export { metadata };
