import {createSlice} from '@reduxjs/toolkit'

export const lastTradeSlice = createSlice({
  name: 'lastTrade',
  initialState: {
    lastTradeTime: Date.now()
  },
  reducers: {
    setTradeOccurred: (state) => {
      state.lastTradeTime = Date.now();
    },
  },
})

// Action creators are generated for each case reducer function
export const {setTradeOccurred} = lastTradeSlice.actions

export default lastTradeSlice.reducer
