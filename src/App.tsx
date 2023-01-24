import { useEffect } from 'react';
import { useLocation } from 'react-router-dom';
import axios from 'axios';


function ImageCell(imgurl: string) {
  return <>

  </>
}

function App() {
  let location = useLocation();
  console.log(location);

  useEffect(() => {
    axios.get(location.pathname + "/test?format=json").then(res => {
      console.log(res)
    })
  }, [location])

  return (
    <div className="absolute w-full h-full top-0 left-0 bottom-0 right-0 bg-slate-200 flex flex-col justify-start items-center overflow-y-auto">
      <div className="grid grid-flow-col auto-cols-max md:auto-cols-min w-full">

      </div>
    </div>

  );
}

export default App;
