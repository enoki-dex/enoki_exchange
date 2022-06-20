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
  maintainAspectRatio: false,
  interaction: {
    intersect: false,
    axis: 'x',
    mode: 'nearest'
  },
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
      },
      ticks: {
        // For a category axis, the val is the index so the lookup via getLabelForValue is needed
        callback: function(val, index) {
          // Hide every 2nd tick label
          return index % 2 === 0 ? val : '';
        }
      }
    }
  },
  plugins: {
    legend: {
      position: 'none'
    },
    tooltip: {
      displayColors: false
    }
  }
};

const PriceHistory = ({lastPrices}) => {

  // console.log(lastPrices);

  const bullish = lastPrices && lastPrices.length && (lastPrices[0].price <= lastPrices[lastPrices.length - 1].price);

  const bigIntToTimestamp = val => {
    val /= BigInt(1e6);
    return Number(val);
  }

  const data = {
    labels: lastPrices.map(last => bigIntToTimestamp(last.time)),
    datasets: [
      {
        data: lastPrices.map(last => last.price),
        borderColor: bullish ? "#00C363" : "#FF6473",
        // cubicInterpolationMode: "default",
        fill: false,
        pointRadius: 0,
        stepped: false,
      }
    ],
  };

  // price: 5.95
  // price_was_lifted: true
  // time: 1655532621059354000n

  return (
    <div className="chart">
      <h3 style={{
        marginLeft: "17px",
        fontWeight: 600,
        marginBottom: "15px",
        marginTop: "5px"
      }}>eICP/eXTC</h3>
      <div className="chart-wrapper">
        <Line options={options} data={data}/>
      </div>
    </div>
  )
}

export default PriceHistory;
