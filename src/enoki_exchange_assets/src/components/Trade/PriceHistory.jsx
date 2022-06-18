import React from "react";

import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Tooltip,
  TimeScale,
  TimeSeriesScale,
} from 'chart.js';
import {Line} from "react-chartjs-2";
import 'chartjs-adapter-moment'

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Tooltip,
  TimeScale,
  TimeSeriesScale,
);

export const options = {
  responsive: true,
  scales: {
    x: {
      type: 'time',
      time: {
        unit: "minute",
        stepSize: 5,
        tooltipFormat: "hh:mm a",
        displayFormats: {
          "minute": "hh:mm a"
        }
      }
    }
  },
  plugins: {
    legend: {
      position: 'none'
    },
  }
};

const PriceHistory = ({lastPrices}) => {

  console.log(lastPrices);

  const bullish = lastPrices && lastPrices.length && (lastPrices[0].price <= lastPrices[lastPrices.length - 1].price);

  const bigIntToTimestamp = val => {
    val /= BigInt(1e6);
    return Number(val);
  }

  const data = {
    labels: lastPrices.map(last => bigIntToTimestamp(last.time)) ,
    datasets: [
      {
        data: lastPrices.map(last => last.price),
        borderColor: bullish ? "#00C363" : "#FF6473",
        cubicInterpolationMode: "monotone",
      }
    ],
  };

  // price: 5.95
  // price_was_lifted: true
  // time: 1655532621059354000n

  return (
    <div className="chart">
      <div className="select">
        <p>eICP/eXTC</p>
        {/*<select name="" id="">*/}
        {/*  <option value="">1D</option>*/}
        {/*  <option value="">2D</option>*/}
        {/*  <option value="">3D</option>*/}
        {/*</select>*/}
      </div>
      <div className="chart-wrapper">
        <Line options={options} data={data} />
      </div>
    </div>
  )
}

export default PriceHistory;
