import { Card, CardHeader, CardBody, CardFooter, Switch } from "@nextui-org/react";
import { ChartOptions } from "chart.js";
import { useState, useEffect, useContext } from "react";
import { Chart } from "react-chartjs-2";
import { StatisticsContext } from "../context";
import { event } from "@tauri-apps/api";
import { Command } from "car-utils";

const options: ChartOptions = {
  responsive: true,
  maintainAspectRatio: true,

  plugins: {
    legend: { labels: { boxWidth: 20 } },
    decimation: {
      enabled: true,
    },

  },
  animation: false,
  scales: {
    x: { reverse: true, display: false, type: "category" },

    "y-axis-temp": {
      type: "linear",
      position: "left",

      max: 100,
      min: -100,
    },
    "y-axis-humidity": {
      type: "linear",
      position: "right",

      min: 0,
      max: 100,
    },

  },
};

const labels = new Array<string>(100).fill("");

/// 温湿度
export function THCard() {
  const { statistics } = useContext(StatisticsContext);

  const [datasets, setDatasets] = useState<[number[], number[]]>([[], []]);

  useEffect(() => {
    setDatasets((datasets) => {
      if (statistics.th !== null) {
        if (datasets[0].length > labels.length) {
          datasets[0].pop();
          datasets[1].pop();
        }

        datasets[0].unshift(statistics.th[0]);
        datasets[1].unshift(statistics.th[1]);
      }

      return datasets;
    });
  }, [statistics]);

  return (
    <Card>
      <CardHeader>
        <Switch
          isSelected={statistics.th != null}
          onValueChange={(enabled) => {
            event.emit("command-server", { kind: "th", enabled } as Command);
          }}
        >
          温湿度传感器
        </Switch>
      </CardHeader>

      <CardBody>
        <Chart
          type="line"
          options={options}
          data={{
            labels,
            datasets: [
              {
                label: "温度 (°C)",
                data: datasets[0],
                borderColor: "rgb(53, 162, 235)",
                // backgroundColor: "rgba(53, 162, 235, 0.5)",
                borderWidth: 1,
                pointRadius: 0,
                tension: 0.4,
                fill: false,
                yAxisID: "y-axis-temp",
              },
              {
                label: "湿度 (%)",
                data: datasets[1],
                borderColor: "red",
                borderWidth: 1,
                // pointStyle: "circle",
                pointRadius: 0,
                // pointHoverRadius: 5,
                tension: 0.4,
                fill: false,
                yAxisID: "y-axis-humidity",
              },
            ],
          }}
          width={400}
          height={150}
        />
      </CardBody>
      <CardFooter className="block">
        <div className="flex justify-around">
          <div>最低气温: {Math.min(...datasets[0]).toFixed(2)}</div>
          {/* <div>最高气温: {statistics.avg.toFixed(2)}</div> */}
          <div>最高气温: {Math.max(...datasets[0]).toFixed(2)}</div>
        </div>
        <div className="flex justify-around">
          <div>最低湿度: {Math.min(...datasets[1]).toFixed(2)}</div>
          {/* <div>平均湿度: {statistics.avg.toFixed(2)}</div> */}
          <div>最高湿度: {Math.max(...datasets[1]).toFixed(2)}</div>
        </div>
      </CardFooter>
    </Card>
  );
}
