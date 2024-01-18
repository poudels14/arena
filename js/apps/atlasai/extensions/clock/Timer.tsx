import { Show, createSignal, onCleanup } from "solid-js";

type TimerState = {
  /**
   * Timestamp of when the timer was started
   */
  startedAt: number;

  /**
   * The total duration of the timer (in milli seconds)
   */
  duration: number;
};

const Timer = (props: { state: TimerState; terminated: boolean }) => {
  const [getRemaining, setRemaining] = createSignal(
    props.state.startedAt + props.state.duration - new Date().getTime()
  );
  const interval = setInterval(() => {
    if (getRemaining() < 0) {
      clearInterval(interval);
    }
    setRemaining(
      props.state.startedAt + props.state.duration - new Date().getTime()
    );
  }, 1000);
  onCleanup(() => clearInterval(interval));
  return (
    <div class="py-12 space-y-8">
      <Show when={!props.terminated && getRemaining() > 0}>
        <div class="flex justify-center text-center font-bold text-5xl text-gray-700">
          <div class="w-20">
            {Math.floor(getRemaining() / 60 / 60_000)
              .toString()
              .padStart(2, "0")}
          </div>
          <div class="-my-1">:</div>
          <div class="w-20">
            {Math.floor(getRemaining() / 60_000)
              .toString()
              .padStart(2, "0")}
          </div>
          <div class="-my-1">:</div>
          <div class="w-20">
            {Math.floor((getRemaining() % 60_000) / 1000)
              .toString()
              .padStart(2, "0")}
          </div>
        </div>
        <div class="flex space-x-4 justify-center text-white">
          <div class="px-6 py-2 bg-red-600 rounded-full cursor-pointer">
            Pause
          </div>
          <div
            class="px-6 py-2 bg-gray-500 rounded-full cursor-pointer text-white"
            onClick={() => clearInterval(interval)}
          >
            Cancel
          </div>
        </div>
      </Show>
      <Show when={props.terminated}>
        <div class="flex justify-center font-semibold text-2xl text-gray-500">
          Timer cancelled
        </div>
      </Show>
      <Show when={getRemaining() < 0}>
        <div class="flex justify-center font-semibold text-2xl text-gray-500">
          Timer expired
        </div>
      </Show>
    </div>
  );
};

export default Timer;
