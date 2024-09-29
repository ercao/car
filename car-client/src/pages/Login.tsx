import { Button, Card, CardBody, CardHeader, Input } from "@nextui-org/react";
import { FC, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

// TODO: 处理连接失败的情况
const Login: FC<{
  loading: boolean;
  setLoading: React.Dispatch<React.SetStateAction<boolean>>;
}> = ({ loading, setLoading }) => {
  const [ipv4, setIpv4] = useState("127.0.0.1");
  const [port, setPort] = useState("5000");

  return (
    <div className="flex justify-center pt-48">
      <Card isBlurred className="w-4/5 md:w-96">
        <CardHeader>树莓派寻迹小车</CardHeader>

        <CardBody className="gap-2">
          <Input
            fullWidth={true}
            required
            label="地址"
            labelPlacement="outside-left"
            placeholder="请输入IP地址"
            value={ipv4}
            onValueChange={setIpv4}
          />

          <Input
            labelPlacement="outside-left"
            required
            label="端口"
            placeholder="请输入端口号"
            value={port}
            onValueChange={setPort}
          />

          <Button
            isLoading={loading}
            color="primary"
            variant="bordered"
            onClick={async () => {
              setLoading(true);
              invoke("connect", { addr: `${ipv4}:${port}` });
            }}
          >
            连接
          </Button>
        </CardBody>
      </Card>
    </div>
  );
};

export default Login;
