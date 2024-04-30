import { JSX, createEffect, createSelector } from "solid-js";
import {
  Router,
  Route,
  useNavigate,
  useMatcher,
  useLocation,
} from "@portal/solid-router";
import { HiSolidCommandLine } from "solid-icons/hi";

const WorkflowTemplates = () => {
  const navigate = useNavigate();
  return (
    <div>
      <div class="flex justify-center">
        <Route
          path="/"
          exact
          component={() => {
            const x = useMatcher(() => "/:path");
            console.log(x());
            console.log("F");
            return (
              <div class="flex-1 max-w-[700px] space-y-4">
                <Template
                  id="1"
                  title="Order from Doordash"
                  description="Order doordash"
                  onClick={(id) => navigate(`/${id}`)}
                />
                <Template
                  id="1"
                  title="Generate QR code"
                  description="This Agent will generate QR code"
                  onClick={(id) => navigate(`/${id}`)}
                />
                <Template
                  id="1"
                  title="Show my active Github PRs"
                  description="Active Github PRs"
                  onClick={(id) => navigate(`/${id}`)}
                />
              </div>
            );
          }}
        />
        <Route
          path="/edit/*"
          component={() => {
            return <div>Nice</div>;
          }}
        />
      </div>
    </div>
  );
};

type TemplateProps = {
  id: string;
  title: string;
  description: string;
  Icon?: JSX.Element;
  onClick: (id: string) => void;
};

const Template = (props: TemplateProps) => {
  return (
    <div
      class="px-3 py-2 flex text-sm border border-gray-100 bg-gray-50 rounded space-x-4 cursor-pointer hover:border-gray-200 hover:bg-gray-100"
      onClick={() => props.onClick(props.id)}
    >
      <div class="py-2 text-indigo-400">
        <HiSolidCommandLine size={30} />
      </div>
      <div class="space-y-1">
        <div class="font-medium">{props.title}</div>
        <div class="text-xs text-gray-700">{props.description}</div>
      </div>
    </div>
  );
};

export default WorkflowTemplates;
