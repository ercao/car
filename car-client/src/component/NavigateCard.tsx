import { Button, Card, CardBody, CardHeader, Slider, Switch } from "@nextui-org/react";
import { event } from "@tauri-apps/api";
import { FC, useContext, useState } from "react";
import { StatisticsContext } from "../context";
import { useHotkeys } from "react-hotkeys-hook";

import { Command, Navigate } from "car-utils";
import { FaArrowDown, FaArrowLeft, FaArrowRight, FaArrowUp } from "react-icons/fa";
import { IoHandLeft } from "react-icons/io5";

/// 导航卡片
/// TODO: 解决同时按两个导航按钮的问题
const NavigateCard: FC = () => {
  const { statistics } = useContext(StatisticsContext);
  const [navigate, setNavigate] = useState<Navigate>("Brake");

  const onPress = (navigate: Navigate) => {
    return () => {
      setNavigate(navigate);
      event.emit("command-server", {
        kind: "navigate",
        navigate,
        speed: statistics.speed_percent,
      } as Command);
    };
  };

  useHotkeys("space", onPress("Brake"));
  useHotkeys("a", onPress("Left"));
  useHotkeys("d", onPress("Right"));
  useHotkeys("w", onPress("Forward"));
  useHotkeys("s", onPress("BackWard"));

  return (
    <Card>
      <CardHeader>方向控制</CardHeader>
      <CardBody>

        <div className="flex justify-around items-center">
          <div className="flex flex-col items-center">
          <div className="inline-grid grid-cols-3  gap-1 grid-rows-3">
            <Button isIconOnly className="row-start-2" onPress={onPress("Left")}>
              <FaArrowLeft />
              {/* 左转<Kbd>A</Kbd> */}
            </Button>
            <Button isIconOnly className="row-start-2 col-start-3" onPress={onPress("Right")}>
              <FaArrowRight />
              {/* 右转<Kbd>D</Kbd> */}
            </Button>
            <Button isIconOnly className="col-start-2 row-start-2 " onPress={onPress("Brake")}>
              <IoHandLeft />
              {/* 刹车
            <Kbd keys={"space"} /> */}
            </Button>
            <Button isIconOnly className="col-start-2" onPress={onPress("Forward")}>
              <FaArrowUp />
              {/* 前进<Kbd>W</Kbd> */}
            </Button>
            <Button isIconOnly className="col-start-2 row-span-3" onPress={onPress("BackWard")}>
              <FaArrowDown />
              {/* 后退<Kbd>S</Kbd> */}
            </Button>
          </div>

          <Switch
            isSelected={statistics.trace}
            onValueChange={(value) => {
              event.emit("command-server", {
                kind: "trace",
                enabled: value,
              } as Command);
            }}
          >
            自动寻迹
          </Switch>
          </div>


          <Slider
            className="inline-flex h-64"
            size="md"
            step={20}
            maxValue={100}
            minValue={0}
            showSteps={true}
            orientation="vertical"
            label="速度"
            marks={[
              { value: 20, label: "一" },
              { value: 40, label: "二" },
              { value: 60, label: "三" },
              { value: 80, label: "四" },
              { value: 100, label: "五" },
            ]}
            value={statistics.speed_percent}
            onChangeEnd={(speed) => {
              event.emit("command-server", {
                kind: "navigate",
                navigate,
                speed,
              } as Command);
            }}
          />
        </div>
      </CardBody>
    </Card>
  );
};

export default NavigateCard;
