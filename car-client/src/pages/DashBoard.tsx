import { FC, useContext, useState } from "react";
import { THCard } from "../component/THCard";
import { event } from "@tauri-apps/api";
import {
  Button,
  Card,
  CardBody,
  Kbd,
  Modal,
  ModalBody,
  ModalContent,
  ModalFooter,
  ModalHeader,
  Slider,
  Switch,
} from "@nextui-org/react";
import { MdOutlineBrightnessHigh, MdOutlineBrightnessLow } from "react-icons/md";
import NavigateCard from "../component/NavigateCard";
import { StatisticsContext } from "../context";
import { useHotkeys } from "react-hotkeys-hook";
import { Command } from "car-utils";
import { FaArrowRotateLeft, FaArrowRotateRight } from "react-icons/fa6";
import { CiCircleQuestion } from "react-icons/ci";

/// 控制面板
const DashBoard: FC<{ addr: string }> = ({ addr }) => {
  const { statistics } = useContext(StatisticsContext);
  const [showHelp, setShowHelp] = useState(false);

  useHotkeys("q", () => {
    if (statistics.servos <= 135) {
      event.emit("command-server", { kind: "servos", angle: statistics.servos + 45 } as Command);
    }
  });

  useHotkeys("e", () => {
    if (statistics.servos >= 0) {
      event.emit("command-server", { kind: "servos", angle: statistics.servos - 45 } as Command);
    }
  });

  return (
    <>
      <div className="w-full mx-auto grid sm:grid-cols-1 md:grid-cols-2 gap-4 space-y-5 p-4">
        <Card className="md:col-span-2">
          <CardBody>
            <div className="flex gap-2 items-center justify-between">
            <div>{addr ? addr : "0.0.0.0:0000"}</div>
            <Button
              size="sm"
              variant="bordered"
              color="danger"
              onClick={async () => {
                await event.emit("close-server");
                await event.emit("close-client");
              }}
            >
              关闭连接
            </Button>
            </div>
          </CardBody>
        </Card>

        <Card>
          <CardBody>
            <div>距离: {statistics.distance} cm</div>
            <div>温度: {statistics.th?.[0]} °c</div>
            <div>湿度: {statistics.th?.[1]} %</div>

            <Switch
              isSelected={statistics.led}
              onValueChange={(enabled) => {
                event.emit("command-server", {
                  kind: "led",
                  enabled,
                } as Command);
              }}
            >
              LED 灯
            </Switch>
          </CardBody>
        </Card>

        <Card>
          <CardBody>
            <Switch
              isSelected={statistics.time_brightness != null}
              onValueChange={(enabled) => {
                event.emit("command-server", {
                  kind: "nixie",
                  enabled,
                  brightness: statistics.time_brightness ?? 0,
                } as Command);
              }}
            >
              显示时间
            </Switch>
            <Slider
              isDisabled={statistics.time_brightness == null}
              size="md"
              step={1}
              color="foreground"
              label="数码管亮度"
              showSteps={true}
              maxValue={7}
              minValue={1}
              value={statistics?.time_brightness ?? 0}
              startContent={<MdOutlineBrightnessLow />}
              endContent={<MdOutlineBrightnessHigh />}
              onChangeEnd={(x) => {
                event.emit("command-server", { kind: "nixie", enabled: true, brightness: x } as Command);
              }}
            />

            <Switch
              isSelected={statistics.distance != null}
              onValueChange={(value) => {
                event.emit("command-server", { kind: "ultrasonic", enabled: value } as Command);
              }}
            >
              超声波测距
            </Switch>

            <div className="space-x-2">
              舵机角度: {statistics.servos}
              &nbsp;
              <Button
                size="sm"
                isIconOnly
                color="primary"
                variant={"bordered"}
                onPress={() => {
                  const angle = statistics.servos;
                  event.emit("command-server", {
                    kind: "servos",
                    angle: angle > 135 ? angle : angle + 45,
                  } as Command);
                }}
              >
                <FaArrowRotateLeft />
              </Button>
              <Button
                isIconOnly
                size="sm"
                color="secondary"
                variant={"bordered"}
                onPress={() => {
                  const angle = statistics.servos;
                  event.emit("command-server", {
                    kind: "servos",
                    angle: angle < 45 ? angle : angle - 45,
                  } as Command);
                }}
              >
                <FaArrowRotateRight />
              </Button>
            </div>
          </CardBody>
        </Card>

        <THCard />
        <NavigateCard />

        {/* TODO: 推流 */}
        {/* Card>
        <CardBody>
          <Image className="w-72" alt="推流" src={`http://${addr}/?action=stream`} />
        </CardBody>
      </Card> */}
      </div>

      <div className="fixed w-full bottom-0">
        <Button className="absolute bottom-4 right-4 shadow-md" isIconOnly onPress={() => setShowHelp(true)}>
          <CiCircleQuestion size={25} />
        </Button>
      </div>
      <Modal placement={"bottom-center"} isOpen={showHelp} onOpenChange={setShowHelp}>
        <ModalContent>
          {(onClose) => (
            <>
              <ModalHeader className="flex flex-col gap-1">快捷键</ModalHeader>
              <ModalBody>
                {[
                  {
                    group: "导航",
                    bindings: [
                      { key: <Kbd>W</Kbd>, description: "前进" },
                      { key: <Kbd>A</Kbd>, description: "左转" },
                      { key: <Kbd>S</Kbd>, description: "后退" },
                      { key: <Kbd>D</Kbd>, description: "右转" },
                      { key: <Kbd keys={["space"]} />, description: "刹车" },
                    ],
                  },

                  {
                    group: "舵机",
                    bindings: [
                      { key: <Kbd>Q</Kbd>, description: "左转" },
                      { key: <Kbd>E</Kbd>, description: "右转" },
                    ],
                  },
                ].map((groups) => {
                  return (
                    <>
                      <div>{groups.group}</div>
                      <div className="grid grid-cols-3">
                        {groups.bindings.map((binding) => {
                          return (
                            <div>
                              {binding.key} {binding.description}
                            </div>
                          );
                        })}
                      </div>
                    </>
                  );
                })}
              </ModalBody>
              <ModalFooter>
                <Button color="danger" variant="light" onPress={onClose}>
                  关闭
                </Button>
              </ModalFooter>
            </>
          )}
        </ModalContent>
      </Modal>
    </>
  );
};

export default DashBoard;
