import {createSlice} from '@reduxjs/toolkit'

const AUTO_VALUE = 0.1;

export const swapSlice = createSlice({
  name: 'swap',
  initialState: {
    slippage: {
      auto: true,
      manualValue: AUTO_VALUE,
      currentValue: AUTO_VALUE,
    }
  },
  reducers: {
    setManual: (state) => {
      state.slippage.auto = false;
      state.slippage.currentValue = state.slippage.manualValue;
    },
    setAuto: (state) => {
      state.slippage.auto = true;
      state.slippage.currentValue = AUTO_VALUE;
    },
    setManualValue: (state, action) => {
      state.slippage.manualValue = action.payload;
      if (!state.slippage.auto) {
        state.slippage.currentValue = action.payload;
      }
    },
  },
})

// Action creators are generated for each case reducer function
export const {setManual, setAuto, setManualValue} = swapSlice.actions

export default swapSlice.reducer
