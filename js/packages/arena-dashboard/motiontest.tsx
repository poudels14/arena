// import { Component, createSignal, Show } from "solid-js";
// // import { animate } from "popmotion";
// import { Motion, Presence } from "@motionone/solid";

// // import { Motion } from "@motionone/solid";
// import { animate } from "motion";

// animate("#box", { transform: "rotate(45deg)" }, { duration: 0.5 });

// const App: Component = () => {
//   const [toggle, setToggle] = createSignal(false);
//   // animate({
//   //   from: 0,
//   //   to: 1,
//   // });

//   // motion("", () => {})
//   return (
//     <div class="container">
//       <Presence exitBeforeEnter>
//       {/* <Show when={toggle()}> */}
//         <Motion.div
//           initial={{ opacity: 0, scale: 0.6 }}
//           animate={{ opacity: 1, scale: 1 }}
//           exit={{ opacity: 0, scale: 0.6 }}
//           transition={{ duration: 0.3 }}
//         >
//           Hello!
//         </Motion.div>
//       {/* </Show> */}
//       </Presence>
//       <button onClick={() => setToggle(!toggle())}>Toggle</button>
//     </div>
//   );
// };

// import { Component, createSignal } from "solid-js";
// import { Motion, Presence } from "@motionone/solid";
// import { Rerun } from "@solid-primitives/keyed";

// const App: Component = () => {
//   const [count, setCount] = createSignal(1);
//   const increment = () => setCount((p) => ++p);
//   return <div class="px-10">
//     <Presence exitBeforeEnter>
//       <Rerun on={count()}>
//         <Motion
//           initial={{ opacity: 0, x: 50 }}
//           animate={{ opacity: 1, x: 0, transition: { delay: 0.05 } }}
//           transition={{ duration: 0.1 }}
//           exit={{ opacity: 0, x: -50 }}
//         >
//           {count()}
//         </Motion>
//       </Rerun>
//     </Presence>
//     <button onClick={increment}>Next</button>
//   </div>;
// };

import { Component, createSignal, mergeProps } from "solid-js";
import { Motion } from "@motionone/solid";
import { Repeat } from "@solid-primitives/range";

const App: Component<{ offset: number; segments: number }> = (props) => {
  props = mergeProps({ offset: 0.09, segments: 8 }, props);
  return (
    <div class="p-32">
      <svg xmlns="http://www.w3.org/2000/svg" width="400" height="200">
        <Repeat times={props.segments}>
          {(i) => (
            <g class="segment">
              <Motion.path
                d="M 94 25 C 94 21.686 96.686 19 100 19 L 100 19 C 103.314 19 106 21.686 106 25 L 106 50 C 106 53.314 103.314 56 100 56 L 100 56 C 96.686 56 94 53.314 94 50 Z"
                style={{
                  transform: "rotate(" + (360 / props.segments) * i + "deg)",
                }}
                animate={{ opacity: [0, 1, 0] }}
                transition={{
                  offset: [0, 0.1, 1],
                  duration: props.offset * props.segments,
                  delay: i * props.offset,
                  repeat: Infinity,
                }}
              />
            </g>
          )}
        </Repeat>
      </svg>
    </div>
  );
};

export default App;
