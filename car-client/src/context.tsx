import { Statistics } from "car-utils";

import { createContext } from "react";

type StatisticsContext = {
  statistics: Statistics;
  setStatistics: React.Dispatch<React.SetStateAction<Statistics>>;
};

export const StatisticsContext = createContext({} as StatisticsContext);
