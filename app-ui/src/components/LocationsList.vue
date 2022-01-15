<template>
  <div v-if="live?.length > 0 || base?.length > 0">
    <table>
      <tr>
        <th>ID</th>
        <th>Name</th>
        <th>Address</th>
      </tr>
      <tr v-for="location in live ?? base" :key="location.id">
        <td>{{ location.id }}</td>
        <td>{{ location.name }}</td>
        <td>{{ location.address ?? '--' }}</td>
      </tr>
    </table>
  </div>
  <div v-else>
    <p>No locations added to the inventory tracking system.</p>
  </div>
</template>
<script>
import { useClientHandle } from '@urql/vue';

export default {
  async setup() {
    const handle = useClientHandle();

    // get the initial locations
    let result = await handle.useQuery({
      query: `
      {
        locations {
          id,
          name,
          address,
        }
      }
      `
    }).executeQuery();
    result = JSON.parse(JSON.stringify(result.data.value.locations));

    // update the locations live
    const handleSubscription = (locations = result, response) => {
      const event = response.locationSubscription;
      if (event.modification === 'CREATE') {
        return [...locations, event.data];
      } else if (event.modification === 'UPDATE') {
        const foundIndex = locations.findIndex(location => location.id === event.data.id);
        locations[foundIndex] = event.data;
        return locations;
      } else if (event.modification === 'DELETE') {
        return locations.filter(location => location.id !== event.data.id);
      }
    }

    // subscribe to location changes
    const subscription = handle.useSubscription({
      query: `
      subscription {
        locationSubscription {
          modification,
          data {
            id,
            name,
            address
          }
        }
      }
      `
    }, handleSubscription);

    // hack to get things working
    return {
      base: result,
      live: subscription.data,
    };
  }
};
</script>