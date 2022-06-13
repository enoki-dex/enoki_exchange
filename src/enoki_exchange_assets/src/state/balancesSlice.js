import {createSlice} from '@reduxjs/toolkit'

export const balancesSlice = createSlice({
  name: 'balances',
  initialState: {
    eICP: 0.0,
    eXTC: 0.0,
  },
  reducers: {
    setBalance: (state, action) => {
      state[action.payload.token] = action.payload.value;
    },
  },
})

// Action creators are generated for each case reducer function
export const {setBalance} = balancesSlice.actions

export default balancesSlice.reducer
