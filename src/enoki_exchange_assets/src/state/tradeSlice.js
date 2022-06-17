import {createSlice} from '@reduxjs/toolkit'

export const tradeSlice = createSlice({
  name: 'trade',
  initialState: {
    allowTaker: true,
  },
  reducers: {
    setOnlyMaker: (state) => {
      state.allowTaker = false;
    },
    setAllowTaker: (state) => {
      state.allowTaker = true;
    },
  },
})

// Action creators are generated for each case reducer function
export const {setOnlyMaker, setAllowTaker} = tradeSlice.actions

export default tradeSlice.reducer
