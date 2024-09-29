import "./index.css";

import { useEffect, useState } from "react";
import {
  CategoryScale,
  Chart as ChartJS,
  LinearScale,
  LineElement,
  PointElement,
  Title,
  Tooltip,
  Filler,
  Legend,
} from "chart.js";
import { event } from "@tauri-apps/api";
import { ToastContainer, toast } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import { useNavigate } from "react-router-dom";
import { NextUIProvider } from "@nextui-org/react";
import DashBoard from "./pages/DashBoard";
import Login from "./pages/Login";
import { Statistics } from "car-utils";
import { StatisticsContext } from "./context";

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Filler, Legend);

function App() {
  const navigate = useNavigate();
  const [isConnected, setIsConnected] = useState(false);
  const [addr, setAddr] = useState<string>("");
  const [connectLoading, setConnectLoading] = useState(false);

  const [statistics, setStatistics] = useState<Statistics>({
    time_brightness: null,
    speed_percent: 20,
    distance: null,
    servos: 90,
    led: false,
    th: null,
    trace: false,
  });

  // 监听事件
  useEffect(() => {
    const listenIds: Promise<event.UnlistenFn>[] = [];
    listenIds.push(
      event.listen<Statistics>("statistics", (event) => {
        console.log(event.payload);
        setStatistics(event.payload);
      })
    );

    // 注册通知事件
    listenIds.push(
      event.listen<string>("notify", (event) => {
        toast(event.payload);
      })
    );
    // 关闭连接事件
    listenIds.push(
      event.listen<string>("close-client", (_event) => {
        setAddr("");
        setIsConnected(false);
        toast("连接关闭");
      })
    );

    // 连接成功事件
    listenIds.push(
      event.listen<string>("connect-client", (event) => {
        console.log(event.payload);
        const payload: { status: true; addr: string } | { status: false; msg: string } = JSON.parse(event.payload);

        if (payload.status) {
          setAddr(payload.addr);
          setIsConnected(true);

          toast(`连接成功：${payload.addr}`);
        } else {
          toast(`连接失败：${payload.msg}`);
        }

        setConnectLoading(false);
      })
    );

    return () => {
      listenIds.forEach((listenId) => {
        listenId.then((f) => f());
      });
    };
  }, []);

  return (
    <NextUIProvider navigate={navigate}>
      <ToastContainer autoClose={1000} pauseOnFocusLoss={false} position="top-right" />
      {!isConnected ? (
        <Login loading={connectLoading} setLoading={setConnectLoading} />
      ) : (
        <StatisticsContext.Provider value={{ statistics, setStatistics }}>
          <DashBoard addr={addr} />
        </StatisticsContext.Provider>
      )} 

      {/*<StatisticsContext.Provider value={{ statistics, setStatistics }}>
        <DashBoard addr={addr} />
      </StatisticsContext.Provider>*/}
    </NextUIProvider>
  );
}

export default App;
