import React, { useEffect } from 'react';
import Helmet from 'react-helmet';
import { useSelector, useDispatch } from 'react-redux'
import useStyles from  './styles';
import { load, updateMessage } from 'redux/modules/traffic';
import Container from '@material-ui/core/Container';
import Typography from '@material-ui/core/Typography';
import { useEventSource, useEventSourceListener } from "@react-nano/use-event-source";
import {
  FlexibleXYPlot,
  YAxis,
  XAxis,
  VerticalGridLines,
  HorizontalGridLines,
  VerticalBarSeries
} from 'react-vis';

function formatFileSize(bytes,decimalPoint) {
  if(bytes < 0) return '';
  if(bytes == 0) return '0 Bytes';
  var k = 1000,
      dm = decimalPoint || 2,
      sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'],
      i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

function Traffic() {
  const loaded = useSelector(state => state.traffic.loaded); 
  const interfaces = useSelector(state => state.traffic.interfaces); 
  const dispatch = useDispatch()

  const styles = useStyles();

  const [eventSource, eventSourceStatus] = useEventSource("/api/events", true);

  useEventSourceListener(eventSource, ['message'], event => {
      dispatch(updateMessage(JSON.parse(event.data)));
  }, [dispatch, updateMessage]);  

  useEffect(() => {
    if (!loaded) {
      dispatch(load());
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []/* run only once */);

  return (
    <>
      <Helmet>
        <title>- Traffic</title>
        <meta name="description" content="Contacts." />
      </Helmet>
      <Container fixed>
        {interfaces.map( ({name, tx, rx}, index) => {
            return <>
              <Typography key={`caption${index}`}>Name: {name}</Typography>
              <div key={`chart${index}`} className={styles.placeholder}>
                <FlexibleXYPlot xType="ordinal" xDistance={100} margin={{left: 100}}>
                  <VerticalGridLines />
                  <HorizontalGridLines />
                  <YAxis tickFormat={v => formatFileSize(v, 2)}/>
                  <XAxis />
                  <VerticalBarSeries className="vertical-bar-series-example" data={tx} />
                  <VerticalBarSeries data={rx} />
                </FlexibleXYPlot>
              </div>
            </>;
          })
        }
        
        <Typography key="eventSourceStatus">{eventSourceStatus === "open" ? "" : "Connecting ..."}</Typography>
      </Container>
    </>
  );
}

export default Traffic;
