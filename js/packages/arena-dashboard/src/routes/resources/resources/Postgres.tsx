import { Form, Input } from "@arena/components/form";

const PostgresConfig = () => {
  return (
    <Form.Nested key="value">
      <div class="space-y-1">
        <label class="block font-medium">Hostname</label>
        <Input name="host" class="w-full" placeholder="Hostname" />
      </div>

      <div class="space-y-1">
        <label class="block font-medium">Port</label>
        <Input name="port" class="w-full" placeholder="Port" />
      </div>

      <div class="space-y-1">
        <label class="block font-medium">Username</label>
        <Input name="username" class="w-full" placeholder="Username" />
      </div>
      <div class="space-y-1">
        <label class="block font-medium">Password</label>
        <Input
          name="password"
          type="password"
          class="w-full"
          placeholder="Password"
        />
      </div>
      <div class="space-y-1">
        <label class="block font-medium">Database</label>
        <Input name="database" class="w-full" placeholder="Database" />
      </div>
    </Form.Nested>
  );
};

export default PostgresConfig;
