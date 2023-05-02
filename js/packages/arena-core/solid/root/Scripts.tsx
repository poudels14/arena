// Credit: Solidjs
import { HydrationScript, isServer, NoHydration } from "solid-js/web";

const isDev = Arena.env.MODE === "development";
const isSSR = Arena.env.ARENA_SSR;

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
                src={"/" + Arena.env.ARENA_ENTRY_CLIENT}
                $ServerOnly
              ></script>
            </>
          ) : (
            <script
              type="module"
              async
              // TODO(sagar): think about how to inject published modules
              src={Arena.env.ARENA_PUBLISHED_ENTRY_CLIENT}
            />
          ))}
      </NoHydration>
    </>
  );
};

export { Scripts };
