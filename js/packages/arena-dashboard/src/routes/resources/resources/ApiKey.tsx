import { Form, Input } from "@arena/components/form";

const ApiKeyConfig = () => {
  return (
    <Form.Nested key="value">
      <div class="space-y-1">
        <label class="block font-medium">API Key</label>
        <Input name="apikey" class="w-full" placeholder="API Key" />
      </div>
    </Form.Nested>
  );
};

export default ApiKeyConfig;
