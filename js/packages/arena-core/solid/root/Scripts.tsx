// Credit: Solidjs
import { HydrationScript, isServer, NoHydration } from "solid-js/web";
import env from "../../env";

const isDev = env.MODE === "development";
const isSSR = env.ARENA_SSR;

const Scripts = () => {
  return (
    <>
      {isSSR && <HydrationScript />}
      <NoHydration>
        {isServer &&
          (isDev ? (
            <>
              <script
                type="module"
                async
                src={"/" + process.env.ARENA_ENTRY_CLIENT}
                $ServerOnly
              ></script>
            </>
          ) : (
            <script
              type="module"
              async
              // TODO(sagar): think about how to inject published modules
              src={process.env.ARENA_PUBLISHED_ENTRY_CLIENT}
            />
          ))}
      </NoHydration>
    </>
  );
};

export { Scripts };
